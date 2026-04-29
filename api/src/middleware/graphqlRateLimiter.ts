/**
 * GraphQL Rate Limiting Middleware
 *
 * Enforces:
 *  - Query complexity limits (weighted field costs)
 *  - Query depth limits
 *  - Per-field limits (max occurrences of a single field)
 *  - User-based request-per-minute limits
 */

import { Request, Response, NextFunction } from "express";
import { parse, DocumentNode, FieldNode, SelectionSetNode, Kind } from "graphql";
import { RateLimiterMemory, RateLimiterRes } from "rate-limiter-flexible";
import { logger } from "../logger";
import { sendError } from "../utils/response";

// ── Configuration ─────────────────────────────────────────────────────────────

export interface GraphQLRateLimitConfig {
  /** Maximum allowed query complexity score (default: 100) */
  maxComplexity: number;
  /** Maximum allowed query depth (default: 7) */
  maxDepth: number;
  /** Maximum times a single field name may appear in a query (default: 10) */
  maxFieldOccurrences: number;
  /** Requests per minute per user/IP (default: 60) */
  requestsPerMinute: number;
  /** Burst allowance per 10 seconds (default: 15) */
  burstLimit: number;
  /** Field cost overrides: fieldName → cost (default cost is 1) */
  fieldCosts?: Record<string, number>;
}

const DEFAULT_CONFIG: GraphQLRateLimitConfig = {
  maxComplexity: 100,
  maxDepth: 7,
  maxFieldOccurrences: 10,
  requestsPerMinute: 60,
  burstLimit: 15,
  fieldCosts: {
    // Expensive fields cost more
    certificates: 5,
    analytics: 5,
    students: 3,
    cohorts: 3,
  },
};

// ── Analysis helpers ──────────────────────────────────────────────────────────

function getDepth(selectionSet: SelectionSetNode | undefined, current = 0): number {
  if (!selectionSet) return current;
  let max = current;
  for (const selection of selectionSet.selections) {
    if (selection.kind === Kind.FIELD) {
      const child = getDepth((selection as FieldNode).selectionSet, current + 1);
      if (child > max) max = child;
    } else if (
      selection.kind === Kind.INLINE_FRAGMENT ||
      selection.kind === Kind.FRAGMENT_SPREAD
    ) {
      // Treat inline fragments as same depth level
      if (selection.kind === Kind.INLINE_FRAGMENT && selection.selectionSet) {
        const child = getDepth(selection.selectionSet, current + 1);
        if (child > max) max = child;
      }
    }
  }
  return max;
}

function analyzeSelectionSet(
  selectionSet: SelectionSetNode | undefined,
  fieldCosts: Record<string, number>,
  fieldCounts: Map<string, number>
): number {
  if (!selectionSet) return 0;
  let complexity = 0;
  for (const selection of selectionSet.selections) {
    if (selection.kind === Kind.FIELD) {
      const field = selection as FieldNode;
      const name = field.name.value;
      const cost = fieldCosts[name] ?? 1;
      complexity += cost;
      fieldCounts.set(name, (fieldCounts.get(name) ?? 0) + 1);
      complexity += analyzeSelectionSet(field.selectionSet, fieldCosts, fieldCounts);
    } else if (selection.kind === Kind.INLINE_FRAGMENT && selection.selectionSet) {
      complexity += analyzeSelectionSet(selection.selectionSet, fieldCosts, fieldCounts);
    }
  }
  return complexity;
}

interface QueryAnalysis {
  complexity: number;
  depth: number;
  maxFieldOccurrences: number;
  mostRepeatedField: string;
}

function analyzeQuery(doc: DocumentNode, fieldCosts: Record<string, number>): QueryAnalysis {
  let totalComplexity = 0;
  let maxDepth = 0;
  const fieldCounts = new Map<string, number>();

  for (const def of doc.definitions) {
    if (
      def.kind === Kind.OPERATION_DEFINITION ||
      def.kind === Kind.FRAGMENT_DEFINITION
    ) {
      const selectionSet = def.selectionSet;
      totalComplexity += analyzeSelectionSet(selectionSet, fieldCosts, fieldCounts);
      const depth = getDepth(selectionSet);
      if (depth > maxDepth) maxDepth = depth;
    }
  }

  let maxOccurrences = 0;
  let mostRepeatedField = "";
  for (const [field, count] of fieldCounts.entries()) {
    if (count > maxOccurrences) {
      maxOccurrences = count;
      mostRepeatedField = field;
    }
  }

  return {
    complexity: totalComplexity,
    depth: maxDepth,
    maxFieldOccurrences: maxOccurrences,
    mostRepeatedField,
  };
}

// ── Per-user rate limiters ────────────────────────────────────────────────────

function makeUserLimiter(rpm: number) {
  return new RateLimiterMemory({ points: rpm, duration: 60, blockDuration: 0 });
}

function makeBurstLimiter(burst: number) {
  return new RateLimiterMemory({ points: burst, duration: 10, blockDuration: 0 });
}

// ── Middleware factory ────────────────────────────────────────────────────────

/**
 * Returns an Express middleware that enforces GraphQL-specific rate limits.
 * Apply this before the GraphQL handler.
 */
export function graphqlRateLimiter(userConfig: Partial<GraphQLRateLimitConfig> = {}) {
  const cfg: GraphQLRateLimitConfig = { ...DEFAULT_CONFIG, ...userConfig };
  const fieldCosts = { ...DEFAULT_CONFIG.fieldCosts, ...userConfig.fieldCosts };

  const mainLimiter = makeUserLimiter(cfg.requestsPerMinute);
  const burstLimiter = makeBurstLimiter(cfg.burstLimit);

  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    // Only apply to POST requests with a GraphQL body
    const body = req.body as { query?: string } | undefined;
    if (!body?.query) {
      next();
      return;
    }

    // ── Parse & analyse the query ─────────────────────────────────────────────
    let doc: DocumentNode;
    try {
      doc = parse(body.query);
    } catch {
      sendError(res, 400, "GRAPHQL_PARSE_ERROR", "Invalid GraphQL query syntax");
      return;
    }

    const analysis = analyzeQuery(doc, fieldCosts);

    // ── Depth check ───────────────────────────────────────────────────────────
    if (analysis.depth > cfg.maxDepth) {
      logger.warn("GraphQL depth limit exceeded", { depth: analysis.depth, max: cfg.maxDepth });
      sendError(
        res,
        400,
        "GRAPHQL_DEPTH_LIMIT",
        `Query depth ${analysis.depth} exceeds maximum allowed depth of ${cfg.maxDepth}.`
      );
      return;
    }

    // ── Complexity check ──────────────────────────────────────────────────────
    if (analysis.complexity > cfg.maxComplexity) {
      logger.warn("GraphQL complexity limit exceeded", {
        complexity: analysis.complexity,
        max: cfg.maxComplexity,
      });
      sendError(
        res,
        400,
        "GRAPHQL_COMPLEXITY_LIMIT",
        `Query complexity ${analysis.complexity} exceeds maximum allowed complexity of ${cfg.maxComplexity}.`
      );
      return;
    }

    // ── Per-field occurrence check ────────────────────────────────────────────
    if (analysis.maxFieldOccurrences > cfg.maxFieldOccurrences) {
      logger.warn("GraphQL field occurrence limit exceeded", {
        field: analysis.mostRepeatedField,
        occurrences: analysis.maxFieldOccurrences,
        max: cfg.maxFieldOccurrences,
      });
      sendError(
        res,
        400,
        "GRAPHQL_FIELD_LIMIT",
        `Field '${analysis.mostRepeatedField}' appears ${analysis.maxFieldOccurrences} times, exceeding the limit of ${cfg.maxFieldOccurrences}.`
      );
      return;
    }

    // ── User-based request rate limit ─────────────────────────────────────────
    const userId: string =
      (req.auth?.sub as string | undefined) ?? req.ip ?? "anonymous";

    try {
      const result = await mainLimiter.consume(userId);
      res.setHeader("X-GraphQL-RateLimit-Limit", cfg.requestsPerMinute);
      res.setHeader("X-GraphQL-RateLimit-Remaining", result.remainingPoints ?? 0);
      res.setHeader(
        "X-GraphQL-RateLimit-Reset",
        Math.ceil((Date.now() + (result.msBeforeNext ?? 0)) / 1000)
      );
      next();
    } catch (err) {
      if (err instanceof RateLimiterRes) {
        // Try burst allowance
        try {
          await burstLimiter.consume(userId);
          logger.info("GraphQL request served from burst allowance", { userId });
          next();
          return;
        } catch (burstErr) {
          if (burstErr instanceof RateLimiterRes) {
            const retryAfter = Math.ceil(burstErr.msBeforeNext / 1000);
            res.setHeader("Retry-After", retryAfter);
            res.setHeader("X-GraphQL-RateLimit-Limit", cfg.requestsPerMinute);
            res.setHeader("X-GraphQL-RateLimit-Remaining", "0");
            logger.warn("GraphQL user rate limit exceeded", { userId, retryAfter });
            sendError(
              res,
              429,
              "GRAPHQL_RATE_LIMIT_EXCEEDED",
              `GraphQL rate limit exceeded. Retry after ${retryAfter}s.`,
              { retryAfter }
            );
            return;
          }
        }
      }
      // Fail open on unexpected errors
      logger.error("GraphQL rate limiter error", { err });
      next();
    }
  };
}
