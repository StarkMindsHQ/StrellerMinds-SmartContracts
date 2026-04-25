/**
 * CDN middleware for static asset delivery.
 *
 * Responsibilities:
 *  - Sets optimal Cache-Control headers for CDN edge caching
 *  - ETag + conditional request support (304 Not Modified)
 *  - Vary headers for correct CDN key construction
 *  - X-Cache header tracking for hit/miss analytics
 *  - Cache invalidation via signed webhook endpoint
 *  - Performance monitoring via Prometheus
 */
import { Request, Response, NextFunction } from "express";
import etag from "etag";
import fresh from "fresh";
import { createHmac, timingSafeEqual } from "crypto";
import { config } from "../config";
import { cdnCacheHits, cdnCacheMisses, cdnInvalidations, assetServeTime } from "../metrics";
import { logger } from "../logger";

// ── In-memory invalidation tag store ─────────────────────────────────────────
// Maps cache key patterns to their invalidation timestamps.
// CDN edge nodes should revalidate any cached response older than this.

const invalidationStore = new Map<string, number>();

export function isInvalidated(cacheKey: string): boolean {
  for (const [pattern, ts] of invalidationStore.entries()) {
    if (cacheKey.startsWith(pattern) && Date.now() - ts < 86_400_000) {
      return true;
    }
  }
  return false;
}

// ── Cache-Control header builder ──────────────────────────────────────────────

interface CacheOptions {
  /** Max age for browser cache (seconds) */
  maxAge?: number;
  /** Max age for CDN/shared cache (seconds) */
  sMaxAge?: number;
  /** Stale-while-revalidate window (seconds) */
  staleWhileRevalidate?: number;
  /** Whether the response is immutable (never changes) */
  immutable?: boolean;
  /** Whether to prevent all caching */
  noStore?: boolean;
  /** Whether to require revalidation before serving stale */
  noCache?: boolean;
}

export function buildCacheControl(opts: CacheOptions): string {
  if (opts.noStore) return "no-store";
  if (opts.noCache) return "no-cache";

  const parts: string[] = ["public"];
  if (opts.maxAge !== undefined) parts.push(`max-age=${opts.maxAge}`);
  if (opts.sMaxAge !== undefined) parts.push(`s-maxage=${opts.sMaxAge}`);
  if (opts.staleWhileRevalidate !== undefined) parts.push(`stale-while-revalidate=${opts.staleWhileRevalidate}`);
  if (opts.immutable) parts.push("immutable");
  return parts.join(", ");
}

// ── Asset type detection ──────────────────────────────────────────────────────

type AssetType = "static" | "api-docs" | "api-response" | "dynamic";

function detectAssetType(path: string): AssetType {
  if (path.startsWith("/api/docs")) return "api-docs";
  if (/\.(js|css|woff2?|ttf|eot|svg|png|jpg|jpeg|gif|ico|webp)(\?|$)/.test(path)) return "static";
  if (path.startsWith("/api/")) return "api-response";
  return "dynamic";
}

// ── CDN headers middleware ────────────────────────────────────────────────────

/**
 * Attaches appropriate Cache-Control, ETag, and CDN headers to responses.
 * Should be applied per-route or globally before route handlers.
 */
export function cdnHeaders(opts: CacheOptions = {}) {
  return (_req: Request, res: Response, next: NextFunction): void => {
    const cacheControl = buildCacheControl(opts);
    res.setHeader("Cache-Control", cacheControl);
    res.setHeader("Vary", "Accept-Encoding, Accept");
    if (config.cdn.origin) {
      res.setHeader("X-CDN-Origin", config.cdn.origin);
    }
    next();
  };
}

/**
 * Full CDN middleware: ETag generation, conditional requests (304),
 * cache hit/miss tracking, and invalidation awareness.
 */
export function cdnMiddleware(req: Request, res: Response, next: NextFunction): void {
  const assetType = detectAssetType(req.path);
  const end = assetServeTime.startTimer({ asset_type: assetType });

  // Intercept res.send to inject ETag and cache headers
  const originalSend = res.send.bind(res);

  res.send = function (body?: unknown): Response {
    // Only process successful GET/HEAD responses
    if (req.method !== "GET" && req.method !== "HEAD") {
      end();
      return originalSend(body);
    }

    const bodyStr = typeof body === "string" ? body : JSON.stringify(body);
    const tag = etag(bodyStr, { weak: true });
    res.setHeader("ETag", tag);

    // Check if invalidated
    const cacheKey = req.path;
    if (isInvalidated(cacheKey)) {
      res.setHeader("Cache-Control", "no-cache");
      res.setHeader("X-Cache-Invalidated", "true");
    }

    // Conditional request — respond 304 if content unchanged
    const reqHeaders = { "if-none-match": req.headers["if-none-match"] ?? "" };
    const resHeaders = { etag: tag };
    if (res.statusCode === 200 && fresh(reqHeaders, resHeaders)) {
      cdnCacheHits.inc({ route: req.path });
      res.setHeader("X-Cache", "HIT");
      end();
      res.status(304).send();
      return res;
    }

    // Determine cache headers by asset type
    if (!res.getHeader("Cache-Control")) {
      switch (assetType) {
        case "static":
          res.setHeader("Cache-Control", buildCacheControl({
            maxAge: config.cdn.maxAge,
            sMaxAge: config.cdn.sMaxAge,
            immutable: true,
          }));
          break;
        case "api-docs":
          res.setHeader("Cache-Control", buildCacheControl({
            maxAge: 3600,
            sMaxAge: config.cdn.sMaxAge,
            staleWhileRevalidate: config.cdn.staleWhileRevalidate,
          }));
          break;
        case "api-response":
          // API responses: short CDN TTL, allow stale-while-revalidate
          res.setHeader("Cache-Control", buildCacheControl({
            maxAge: 0,
            sMaxAge: 30,
            staleWhileRevalidate: config.cdn.staleWhileRevalidate,
          }));
          break;
        default:
          res.setHeader("Cache-Control", "no-store");
      }
    }

    cdnCacheMisses.inc({ route: req.path });
    res.setHeader("X-Cache", "MISS");
    res.setHeader("X-Content-Type-Options", "nosniff");
    end();
    return originalSend(body);
  };

  next();
}

// ── Cache invalidation ────────────────────────────────────────────────────────

/**
 * Validates the HMAC-SHA256 signature on an invalidation request.
 * Signature = HMAC-SHA256(secret, body_string)
 */
export function validateInvalidationSignature(
  body: string,
  signature: string
): boolean {
  try {
    const expected = createHmac("sha256", config.cdn.invalidationSecret)
      .update(body)
      .digest("hex");
    const expectedBuf = Buffer.from(`sha256=${expected}`);
    const providedBuf = Buffer.from(signature);
    if (expectedBuf.length !== providedBuf.length) return false;
    return timingSafeEqual(expectedBuf, providedBuf);
  } catch {
    return false;
  }
}

/**
 * Registers a cache invalidation for a path pattern.
 * All cached responses whose path starts with `pattern` will be treated as stale.
 */
export function invalidateCache(pattern: string): void {
  invalidationStore.set(pattern, Date.now());
  cdnInvalidations.inc({ pattern });
  logger.info("Cache invalidated", { pattern });
}

/**
 * Returns all active invalidation entries.
 */
export function getInvalidationStore(): Record<string, number> {
  return Object.fromEntries(invalidationStore.entries());
}
