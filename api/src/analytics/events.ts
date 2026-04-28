/**
 * events.ts
 *
 * Typed GA4 event factory functions for the StrellerMinds Certificate API.
 *
 * Each function returns a Ga4Event ready to pass to sendGa4Event().
 * Keeping events in one place makes it easy to audit what is tracked
 * and to update event schemas as the product evolves.
 *
 * Naming convention follows GA4 recommended event names where applicable,
 * with custom snake_case names for domain-specific events.
 */

import { sendGa4Event } from "./ga4Client";
import type { Ga4Event } from "./ga4Client";

// ── Event name constants (single source of truth) ────────────────────────────

export const EVENT_NAMES = {
  AUTH_TOKEN_ISSUED: "auth_token_issued",
  CERTIFICATE_VERIFIED: "certificate_verified",
  CERTIFICATE_DETAIL_FETCHED: "certificate_detail_fetched",
  REVOCATION_CHECKED: "revocation_checked",
  ANALYTICS_QUERIED: "analytics_queried",
  STUDENT_CERTS_LISTED: "student_certs_listed",
  API_RATE_LIMITED: "api_rate_limited",
} as const;

export type EventName = (typeof EVENT_NAMES)[keyof typeof EVENT_NAMES];

// ── Event factories ───────────────────────────────────────────────────────────

/**
 * Fired when an API consumer successfully obtains a JWT access token.
 * Maps to a GA4 "sign_up / login" conversion.
 */
export function makeAuthTokenIssuedEvent(scope: string[]): Ga4Event {
  return {
    name: EVENT_NAMES.AUTH_TOKEN_ISSUED,
    params: {
      scope: scope.join(","),
      engagement_time_msec: 1,
    },
  };
}

/**
 * Fired for every certificate verification attempt (public endpoint).
 * This is the primary conversion event.
 *
 * @param certId  - Public certificate identifier (not PII)
 * @param result  - "valid" | "invalid" | "not_found" | "error"
 */
export function makeCertificateVerifiedEvent(
  certId: string,
  result: "valid" | "invalid" | "not_found" | "error"
): Ga4Event {
  return {
    name: EVENT_NAMES.CERTIFICATE_VERIFIED,
    params: {
      certificate_id: certId.slice(0, 16), // truncate to avoid long custom dim values
      verification_result: result,
      engagement_time_msec: 1,
    },
  };
}

/**
 * Fired when an authenticated consumer fetches full certificate details.
 * Equivalent to a "page_view" for this resource.
 */
export function makeCertificateDetailFetchedEvent(certId: string): Ga4Event {
  return {
    name: EVENT_NAMES.CERTIFICATE_DETAIL_FETCHED,
    params: {
      certificate_id: certId.slice(0, 16),
      engagement_time_msec: 1,
    },
  };
}

/**
 * Fired when a revocation record is requested.
 *
 * @param certId - Public certificate identifier
 * @param found  - Whether a revocation record was found
 */
export function makeRevocationCheckedEvent(
  certId: string,
  found: boolean
): Ga4Event {
  return {
    name: EVENT_NAMES.REVOCATION_CHECKED,
    params: {
      certificate_id: certId.slice(0, 16),
      revocation_found: found,
      engagement_time_msec: 1,
    },
  };
}

/**
 * Fired when the aggregate analytics endpoint is hit by an authenticated consumer.
 */
export function makeAnalyticsQueriedEvent(): Ga4Event {
  return {
    name: EVENT_NAMES.ANALYTICS_QUERIED,
    params: { engagement_time_msec: 1 },
  };
}

/**
 * Fired when student certificate list is retrieved.
 *
 * @param studentAddress - Stellar public key (not PII under GDPR — it's a pseudonym)
 */
export function makeStudentCertsListedEvent(studentAddress: string): Ga4Event {
  return {
    name: EVENT_NAMES.STUDENT_CERTS_LISTED,
    params: {
      // Only the first 8 chars — enough for cohort analysis, not re-identification
      student_prefix: studentAddress.slice(0, 8),
      engagement_time_msec: 1,
    },
  };
}

/**
 * Fired when a request is rate-limited.  Useful for abuse detection dashboards.
 *
 * @param endpoint - The route path that was rate-limited
 */
export function makeApiRateLimitedEvent(endpoint: string): Ga4Event {
  return {
    name: EVENT_NAMES.API_RATE_LIMITED,
    params: {
      endpoint,
      engagement_time_msec: 1,
    },
  };
}

// ── Convenience fire helpers ──────────────────────────────────────────────────
// These wrap factory + sendGa4Event for one-liner usage in route handlers.

export function trackAuthTokenIssued(
  clientId: string,
  scope: string[],
  optOut = false
): void {
  sendGa4Event(clientId, makeAuthTokenIssuedEvent(scope), optOut);
}

export function trackCertificateVerified(
  clientId: string,
  certId: string,
  result: "valid" | "invalid" | "not_found" | "error",
  optOut = false
): void {
  sendGa4Event(clientId, makeCertificateVerifiedEvent(certId, result), optOut);
}

export function trackCertificateDetailFetched(
  clientId: string,
  certId: string,
  optOut = false
): void {
  sendGa4Event(clientId, makeCertificateDetailFetchedEvent(certId), optOut);
}

export function trackRevocationChecked(
  clientId: string,
  certId: string,
  found: boolean,
  optOut = false
): void {
  sendGa4Event(clientId, makeRevocationCheckedEvent(certId, found), optOut);
}

export function trackAnalyticsQueried(
  clientId: string,
  optOut = false
): void {
  sendGa4Event(clientId, makeAnalyticsQueriedEvent(), optOut);
}

export function trackStudentCertsListed(
  clientId: string,
  studentAddress: string,
  optOut = false
): void {
  sendGa4Event(clientId, makeStudentCertsListedEvent(studentAddress), optOut);
}
