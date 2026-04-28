/**
 * events.test.ts
 *
 * Unit tests for all GA4 event factory functions and convenience fire helpers.
 */

// ── Env setup ─────────────────────────────────────────────────────────────────
process.env.GA4_MEASUREMENT_ID = "G-TESTID1234";
process.env.GA4_API_SECRET = "test-api-secret";
process.env.GA4_ENABLED = "true";
process.env.GA4_DEBUG = "false";

// ── Mock the GA4 client so no real HTTP calls are made ────────────────────────
const mockSendGa4Event = jest.fn();
jest.mock("../ga4Client", () => ({
  sendGa4Event: (...args: unknown[]) => mockSendGa4Event(...args),
  anonymizeClientId: (raw: string) => `anon-${raw}`,
}));

jest.mock("../../logger", () => ({
  logger: { debug: jest.fn(), info: jest.fn(), warn: jest.fn(), error: jest.fn() },
}));

import {
  EVENT_NAMES,
  makeAuthTokenIssuedEvent,
  makeCertificateVerifiedEvent,
  makeCertificateDetailFetchedEvent,
  makeRevocationCheckedEvent,
  makeAnalyticsQueriedEvent,
  makeStudentCertsListedEvent,
  makeApiRateLimitedEvent,
  trackAuthTokenIssued,
  trackCertificateVerified,
  trackCertificateDetailFetched,
  trackRevocationChecked,
  trackAnalyticsQueried,
  trackStudentCertsListed,
} from "../events";

beforeEach(() => {
  mockSendGa4Event.mockReset();
});

// ── EVENT_NAMES constants ─────────────────────────────────────────────────────

describe("EVENT_NAMES", () => {
  it("has stable snake_case values", () => {
    expect(EVENT_NAMES.AUTH_TOKEN_ISSUED).toBe("auth_token_issued");
    expect(EVENT_NAMES.CERTIFICATE_VERIFIED).toBe("certificate_verified");
    expect(EVENT_NAMES.CERTIFICATE_DETAIL_FETCHED).toBe("certificate_detail_fetched");
    expect(EVENT_NAMES.REVOCATION_CHECKED).toBe("revocation_checked");
    expect(EVENT_NAMES.ANALYTICS_QUERIED).toBe("analytics_queried");
    expect(EVENT_NAMES.STUDENT_CERTS_LISTED).toBe("student_certs_listed");
    expect(EVENT_NAMES.API_RATE_LIMITED).toBe("api_rate_limited");
  });
});

// ── Factory functions ─────────────────────────────────────────────────────────

describe("makeAuthTokenIssuedEvent", () => {
  it("returns correct event name", () => {
    const e = makeAuthTokenIssuedEvent(["verify", "read"]);
    expect(e.name).toBe(EVENT_NAMES.AUTH_TOKEN_ISSUED);
  });

  it("joins scope array into a comma-separated string", () => {
    const e = makeAuthTokenIssuedEvent(["verify", "read", "write"]);
    expect(e.params?.scope).toBe("verify,read,write");
  });

  it("includes engagement_time_msec", () => {
    const e = makeAuthTokenIssuedEvent(["verify"]);
    expect(e.params?.engagement_time_msec).toBe(1);
  });
});

describe("makeCertificateVerifiedEvent", () => {
  const CERT_ID = "a".repeat(64);

  it("returns correct event name", () => {
    const e = makeCertificateVerifiedEvent(CERT_ID, "valid");
    expect(e.name).toBe(EVENT_NAMES.CERTIFICATE_VERIFIED);
  });

  it("truncates certificate_id to 16 chars", () => {
    const e = makeCertificateVerifiedEvent(CERT_ID, "valid");
    expect((e.params?.certificate_id as string).length).toBe(16);
  });

  it.each(["valid", "invalid", "not_found", "error"] as const)(
    "accepts result='%s'",
    (result) => {
      const e = makeCertificateVerifiedEvent(CERT_ID, result);
      expect(e.params?.verification_result).toBe(result);
    }
  );
});

describe("makeCertificateDetailFetchedEvent", () => {
  it("returns correct event name with truncated cert ID", () => {
    const e = makeCertificateDetailFetchedEvent("b".repeat(64));
    expect(e.name).toBe(EVENT_NAMES.CERTIFICATE_DETAIL_FETCHED);
    expect((e.params?.certificate_id as string).length).toBe(16);
  });
});

describe("makeRevocationCheckedEvent", () => {
  it("sets revocation_found=true when record found", () => {
    const e = makeRevocationCheckedEvent("c".repeat(64), true);
    expect(e.name).toBe(EVENT_NAMES.REVOCATION_CHECKED);
    expect(e.params?.revocation_found).toBe(true);
  });

  it("sets revocation_found=false when not found", () => {
    const e = makeRevocationCheckedEvent("c".repeat(64), false);
    expect(e.params?.revocation_found).toBe(false);
  });
});

describe("makeAnalyticsQueriedEvent", () => {
  it("returns correct event name", () => {
    const e = makeAnalyticsQueriedEvent();
    expect(e.name).toBe(EVENT_NAMES.ANALYTICS_QUERIED);
  });

  it("includes engagement_time_msec", () => {
    const e = makeAnalyticsQueriedEvent();
    expect(e.params?.engagement_time_msec).toBe(1);
  });
});

describe("makeStudentCertsListedEvent", () => {
  it("returns correct event name", () => {
    const e = makeStudentCertsListedEvent("GABCDE12345678901234");
    expect(e.name).toBe(EVENT_NAMES.STUDENT_CERTS_LISTED);
  });

  it("only stores first 8 chars of student address", () => {
    const e = makeStudentCertsListedEvent("GABCDE12345678901234");
    expect((e.params?.student_prefix as string).length).toBe(8);
    expect(e.params?.student_prefix).toBe("GABCDE12");
  });
});

describe("makeApiRateLimitedEvent", () => {
  it("returns correct event name and endpoint", () => {
    const e = makeApiRateLimitedEvent("/api/v1/certificates/:id/verify");
    expect(e.name).toBe(EVENT_NAMES.API_RATE_LIMITED);
    expect(e.params?.endpoint).toBe("/api/v1/certificates/:id/verify");
  });
});

// ── Convenience fire helpers ──────────────────────────────────────────────────

describe("trackAuthTokenIssued", () => {
  it("calls sendGa4Event with correct args when opt-in", () => {
    trackAuthTokenIssued("client-1", ["verify"], false);
    expect(mockSendGa4Event).toHaveBeenCalledTimes(1);
    const [clientId, event, optOut] = mockSendGa4Event.mock.calls[0] as [string, { name: string }, boolean];
    expect(clientId).toBe("client-1");
    expect(event.name).toBe(EVENT_NAMES.AUTH_TOKEN_ISSUED);
    expect(optOut).toBe(false);
  });

  it("passes optOut=true to sendGa4Event when opted out", () => {
    trackAuthTokenIssued("client-2", ["verify"], true);
    const [, , optOut] = mockSendGa4Event.mock.calls[0] as [string, unknown, boolean];
    expect(optOut).toBe(true);
  });
});

describe("trackCertificateVerified", () => {
  it("forwards all params to sendGa4Event", () => {
    trackCertificateVerified("cid", "a".repeat(64), "valid", false);
    expect(mockSendGa4Event).toHaveBeenCalledTimes(1);
    const [clientId, event] = mockSendGa4Event.mock.calls[0] as [string, { name: string; params: Record<string, unknown> }];
    expect(clientId).toBe("cid");
    expect(event.name).toBe(EVENT_NAMES.CERTIFICATE_VERIFIED);
    expect(event.params.verification_result).toBe("valid");
  });
});

describe("trackCertificateDetailFetched", () => {
  it("calls sendGa4Event with cert detail event", () => {
    trackCertificateDetailFetched("cid", "b".repeat(64), false);
    const [, event] = mockSendGa4Event.mock.calls[0] as [string, { name: string }];
    expect(event.name).toBe(EVENT_NAMES.CERTIFICATE_DETAIL_FETCHED);
  });
});

describe("trackRevocationChecked", () => {
  it("calls sendGa4Event with revocation event", () => {
    trackRevocationChecked("cid", "c".repeat(64), true, false);
    const [, event] = mockSendGa4Event.mock.calls[0] as [string, { name: string; params: Record<string, unknown> }];
    expect(event.name).toBe(EVENT_NAMES.REVOCATION_CHECKED);
    expect(event.params.revocation_found).toBe(true);
  });
});

describe("trackAnalyticsQueried", () => {
  it("calls sendGa4Event with analytics queried event", () => {
    trackAnalyticsQueried("cid", false);
    const [, event] = mockSendGa4Event.mock.calls[0] as [string, { name: string }];
    expect(event.name).toBe(EVENT_NAMES.ANALYTICS_QUERIED);
  });
});

describe("trackStudentCertsListed", () => {
  it("calls sendGa4Event with student certs listed event", () => {
    trackStudentCertsListed("cid", "GABCDE12345678901234", false);
    const [, event] = mockSendGa4Event.mock.calls[0] as [string, { name: string; params: Record<string, unknown> }];
    expect(event.name).toBe(EVENT_NAMES.STUDENT_CERTS_LISTED);
    expect(event.params.student_prefix).toBe("GABCDE12");
  });
});
