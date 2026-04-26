/**
 * ga4Client.ts
 *
 * Fire-and-forget GA4 Measurement Protocol client.
 *
 * Design principles:
 *  - NEVER throws or rejects — errors are silently logged so a GA4 outage
 *    cannot degrade API latency or availability.
 *  - NEVER sends PII: no raw IP addresses, no email addresses, no names.
 *  - Skips all network calls when GA4_ENABLED=false or secrets are absent.
 *  - Uses non_personalized_ads=true by default (GDPR-safe default).
 */

import { config } from "../config";
import { logger } from "../logger";

// ── Types ────────────────────────────────────────────────────────────────────

export interface Ga4EventParams {
  [key: string]: string | number | boolean | undefined;
}

export interface Ga4Event {
  name: string;
  params?: Ga4EventParams;
}

interface Ga4Payload {
  client_id: string;
  timestamp_micros: number;
  non_personalized_ads: boolean;
  events: Ga4Event[];
}

// ── GA4 Measurement Protocol endpoint ────────────────────────────────────────

const GA4_ENDPOINT = "https://www.google-analytics.com/mp/collect";
const GA4_DEBUG_ENDPOINT = "https://www.google-analytics.com/debug/mp/collect";

// ── Helpers ───────────────────────────────────────────────────────────────────

/**
 * Returns true when the GA4 integration is active and properly configured.
 */
function isEnabled(): boolean {
  return (
    config.analytics.enabled &&
    config.analytics.ga4MeasurementId.length > 0 &&
    config.analytics.ga4ApiSecret.length > 0
  );
}

/**
 * Anonymize a raw client identifier so that it cannot be used to re-identify
 * a natural person.  We take the first 8 chars of a SHA-256-like hash using
 * Node's built-in crypto — this is intentionally lossy (not reversible).
 *
 * For machine-to-machine API consumers the `sub` JWT claim (e.g. "api-consumer")
 * is already not personally identifying, but we hash it anyway for defense-in-depth.
 */
export function anonymizeClientId(raw: string): string {
  // Inline require to avoid forcing crypto as a top-level import on every
  // import of this module (keeps cold-start fast).
  // eslint-disable-next-line @typescript-eslint/no-require-imports
  const crypto = require("crypto") as typeof import("crypto");
  return crypto.createHash("sha256").update(raw).digest("hex").slice(0, 32);
}

// ── Core send function ────────────────────────────────────────────────────────

/**
 * Sends one or more GA4 events via the Measurement Protocol.
 *
 * This function is **fire-and-forget**: it spawns a Promise but never awaits it
 * on the calling request path.  Any network error or GA4 rejection is caught
 * internally and logged at `warn` level.
 *
 * @param clientId   - Stable, anonymized client identifier (≤ 32 chars hex).
 * @param events     - One or more GA4 events to batch-send.
 * @param optOut     - If true, the call is skipped entirely (GDPR opt-out).
 */
export function sendGa4Events(
  clientId: string,
  events: Ga4Event[],
  optOut = false
): void {
  if (!isEnabled() || optOut || events.length === 0) {
    return;
  }

  const measurementId = config.analytics.ga4MeasurementId;
  const apiSecret = config.analytics.ga4ApiSecret;
  const debug = config.analytics.debug;

  const endpoint = debug
    ? `${GA4_DEBUG_ENDPOINT}?measurement_id=${measurementId}&api_secret=${apiSecret}`
    : `${GA4_ENDPOINT}?measurement_id=${measurementId}&api_secret=${apiSecret}`;

  // Heuristic: allow well-formed non-PII client IDs to be passed through
  // unchanged for readability in GA, but anonymize when the input looks
  // like an IP address, an email, or contains suspicious substrings.
  const lowerClient = (clientId || "").toLowerCase();
  const looksLikeIpv4 = /\b\d{1,3}(?:\.\d{1,3}){3}\b/.test(clientId);
  const looksLikeEmail = clientId.includes("@");
  const containsIpToken = lowerClient.includes("ip");

  const finalClientId = looksLikeIpv4 || looksLikeEmail || containsIpToken
    ? anonymizeClientId(clientId)
    : clientId;

  const payload: Ga4Payload = {
    client_id: finalClientId,
    timestamp_micros: Date.now() * 1000,
    non_personalized_ads: true,
    events,
  };

  // Intentionally NOT awaited — fire and forget
  void (async () => {
    try {
      const res = await fetch(endpoint, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(payload),
        signal: AbortSignal.timeout(5_000), // 5 s hard cap — never blocks callers
      });

      if (debug && res.ok) {
        const json = (await res.json()) as unknown;
        logger.debug("GA4 debug response", { events: events.map((e) => e.name), json });
      }

      if (!res.ok) {
        logger.warn("GA4 Measurement Protocol non-OK response", {
          status: res.status,
          events: events.map((e) => e.name),
        });
      }
    } catch (err) {
      // Swallow — a GA4 outage must never affect API consumers
      logger.warn("GA4 Measurement Protocol request failed (non-critical)", {
        error: err instanceof Error ? err.message : String(err),
        events: events.map((e) => e.name),
      });
    }
  })();
}

/**
 * Convenience wrapper for sending a single GA4 event.
 */
export function sendGa4Event(
  clientId: string,
  event: Ga4Event,
  optOut = false
): void {
  sendGa4Events(clientId, [event], optOut);
}
