/**
 * ga4Client.test.ts
 *
 * Unit tests for the GA4 Measurement Protocol client.
 * Verifies fire-and-forget behavior, GDPR safeguards, and env-toggle logic.
 */

// ── Setup: must be done before importing the module under test ────────────────

// Freeze a stable measurement ID / secret so config reads are predictable
process.env.GA4_MEASUREMENT_ID = "G-TESTID1234";
process.env.GA4_API_SECRET = "test-api-secret";
process.env.GA4_ENABLED = "true";
process.env.GA4_DEBUG = "false";

// ── Mocks ─────────────────────────────────────────────────────────────────────

const mockFetch = jest.fn();
global.fetch = mockFetch as unknown as typeof fetch;

// Silence winston during tests
jest.mock("../../logger", () => ({
  logger: { debug: jest.fn(), info: jest.fn(), warn: jest.fn(), error: jest.fn() },
}));

// ── Re-import after env is set ────────────────────────────────────────────────
// We use jest.resetModules() in beforeEach so each test gets a fresh config.

describe("ga4Client", () => {
  let sendGa4Event: (typeof import("../ga4Client"))["sendGa4Event"];
  let sendGa4Events: (typeof import("../ga4Client"))["sendGa4Events"];
  let anonymizeClientId: (typeof import("../ga4Client"))["anonymizeClientId"];

  beforeEach(async () => {
    jest.resetModules();
    mockFetch.mockReset();
    // Default success response
    mockFetch.mockResolvedValue({ ok: true, json: async () => ({}) } as Response);

    // Re-import with fresh module cache
    const mod = await import("../ga4Client");
    sendGa4Event = mod.sendGa4Event;
    sendGa4Events = mod.sendGa4Events;
    anonymizeClientId = mod.anonymizeClientId;
  });

  // ── anonymizeClientId ────────────────────────────────────────────────────────

  describe("anonymizeClientId", () => {
    it("returns a 32-char hex string", () => {
      const id = anonymizeClientId("some-consumer");
      expect(id).toHaveLength(32);
      expect(id).toMatch(/^[0-9a-f]+$/);
    });

    it("is deterministic for the same input", () => {
      expect(anonymizeClientId("abc")).toBe(anonymizeClientId("abc"));
    });

    it("produces different outputs for different inputs", () => {
      expect(anonymizeClientId("user-a")).not.toBe(anonymizeClientId("user-b"));
    });

    it("never includes raw PII", () => {
      const raw = "user@example.com";
      const id = anonymizeClientId(raw);
      expect(id).not.toContain("user");
      expect(id).not.toContain("example");
    });
  });

  // ── sendGa4Event — happy path ────────────────────────────────────────────────

  describe("sendGa4Event", () => {
    it("calls fetch with the correct Measurement Protocol URL", async () => {
      sendGa4Event("client-abc", { name: "test_event" });

      // Fire-and-forget: give the microtask queue a tick to flush
      await new Promise((r) => setTimeout(r, 50));

      expect(mockFetch).toHaveBeenCalledTimes(1);
      const [url] = mockFetch.mock.calls[0] as [string, RequestInit];
      expect(url).toContain("https://www.google-analytics.com/mp/collect");
      expect(url).toContain("measurement_id=G-TESTID1234");
      expect(url).toContain("api_secret=test-api-secret");
    });

    it("sends correct JSON payload with required fields", async () => {
      sendGa4Event("client-xyz", { name: "cert_verified", params: { result: "valid" } });
      await new Promise((r) => setTimeout(r, 50));

      const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
      const body = JSON.parse(init.body as string) as {
        client_id: string;
        non_personalized_ads: boolean;
        timestamp_micros: number;
        events: Array<{ name: string }>;
      };

      expect(body.client_id).toBe("client-xyz");
      expect(body.non_personalized_ads).toBe(true);
      expect(typeof body.timestamp_micros).toBe("number");
      expect(body.events[0].name).toBe("cert_verified");
    });

    it("never includes IP address in the payload", async () => {
      sendGa4Event("client-ip-test", { name: "test_event" });
      await new Promise((r) => setTimeout(r, 50));

      const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
      const bodyStr = init.body as string;
      // Ensure no IP-like patterns
      expect(bodyStr).not.toMatch(/\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3}/);
      expect(bodyStr).not.toContain("ip");
      expect(bodyStr).not.toContain("user_ip");
    });
  });

  // ── Opt-out / GDPR ───────────────────────────────────────────────────────────

  describe("opt-out (GDPR)", () => {
    it("skips fetch when optOut=true", async () => {
      sendGa4Event("client-abc", { name: "should_not_send" }, true);
      await new Promise((r) => setTimeout(r, 50));
      expect(mockFetch).not.toHaveBeenCalled();
    });

    it("skips fetch when events array is empty", async () => {
      sendGa4Events("client-abc", []);
      await new Promise((r) => setTimeout(r, 50));
      expect(mockFetch).not.toHaveBeenCalled();
    });
  });

  // ── GA4_ENABLED=false ────────────────────────────────────────────────────────

  describe("GA4_ENABLED=false", () => {
    beforeEach(async () => {
      process.env.GA4_ENABLED = "false";
      jest.resetModules();
      const mod = await import("../ga4Client");
      sendGa4Event = mod.sendGa4Event;
    });

    afterEach(() => {
      process.env.GA4_ENABLED = "true";
    });

    it("does not call fetch when GA4_ENABLED is false", async () => {
      sendGa4Event("client-abc", { name: "test_disabled" });
      await new Promise((r) => setTimeout(r, 50));
      expect(mockFetch).not.toHaveBeenCalled();
    });
  });

  // ── Missing credentials ───────────────────────────────────────────────────────

  describe("missing credentials", () => {
    beforeEach(async () => {
      process.env.GA4_MEASUREMENT_ID = "";
      jest.resetModules();
      const mod = await import("../ga4Client");
      sendGa4Event = mod.sendGa4Event;
    });

    afterEach(() => {
      process.env.GA4_MEASUREMENT_ID = "G-TESTID1234";
    });

    it("does not call fetch when Measurement ID is empty", async () => {
      sendGa4Event("client-abc", { name: "test_no_creds" });
      await new Promise((r) => setTimeout(r, 50));
      expect(mockFetch).not.toHaveBeenCalled();
    });
  });

  // ── Fire-and-forget: network failures ────────────────────────────────────────

  describe("fire-and-forget error handling", () => {
    it("does NOT throw when fetch rejects", async () => {
      mockFetch.mockRejectedValue(new Error("Network failure"));

      // Must not throw — fire-and-forget
      expect(() => {
        sendGa4Event("client-abc", { name: "test_network_error" });
      }).not.toThrow();

      // Allow microtask queue to drain — still no unhandled rejection
      await new Promise((r) => setTimeout(r, 100));
    });

    it("does NOT throw when GA4 returns non-OK status", async () => {
      mockFetch.mockResolvedValue({ ok: false, status: 400, json: async () => ({}) } as Response);

      expect(() => {
        sendGa4Event("client-abc", { name: "test_bad_response" });
      }).not.toThrow();

      await new Promise((r) => setTimeout(r, 50));
      // fetch was called but failure was silently logged
      expect(mockFetch).toHaveBeenCalledTimes(1);
    });

    it("does NOT throw when AbortSignal times out", async () => {
      mockFetch.mockRejectedValue(new DOMException("Aborted", "AbortError"));

      expect(() => {
        sendGa4Event("client-abc", { name: "test_timeout" });
      }).not.toThrow();

      await new Promise((r) => setTimeout(r, 100));
    });
  });

  // ── Batch send ───────────────────────────────────────────────────────────────

  describe("sendGa4Events (batch)", () => {
    it("sends multiple events in a single HTTP request", async () => {
      sendGa4Events("client-batch", [
        { name: "event_one" },
        { name: "event_two", params: { value: 42 } },
      ]);
      await new Promise((r) => setTimeout(r, 50));

      expect(mockFetch).toHaveBeenCalledTimes(1);
      const [, init] = mockFetch.mock.calls[0] as [string, RequestInit];
      const body = JSON.parse(init.body as string) as { events: Array<{ name: string }> };
      expect(body.events).toHaveLength(2);
      expect(body.events[0].name).toBe("event_one");
      expect(body.events[1].name).toBe("event_two");
    });
  });
});
