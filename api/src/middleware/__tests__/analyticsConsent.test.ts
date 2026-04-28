/**
 * analyticsConsent.test.ts
 *
 * Unit tests for the GDPR analytics consent middleware.
 */

import { Request, Response, NextFunction } from "express";
import { analyticsConsent } from "../../middleware/analyticsConsent";

// ── Helpers ───────────────────────────────────────────────────────────────────

function makeReq(consentHeader?: string): Partial<Request> {
  return {
    headers: consentHeader ? { "x-analytics-consent": consentHeader } : {},
  };
}

function makeRes(): { setHeader: jest.Mock; headers: Record<string, string> } {
  const headers: Record<string, string> = {};
  return {
    setHeader: jest.fn((key: string, value: string) => {
      headers[key] = value;
    }),
    headers,
  };
}

function makeNext(): NextFunction {
  return jest.fn();
}

// ── Tests ─────────────────────────────────────────────────────────────────────

describe("analyticsConsent middleware", () => {
  describe("X-Analytics-Consent: denied", () => {
    it("sets req.analyticsOptOut = true", () => {
      const req = makeReq("denied") as Request;
      const res = makeRes() as unknown as Response;
      const next = makeNext();

      analyticsConsent(req, res, next);

      expect(req.analyticsOptOut).toBe(true);
    });

    it("still calls next()", () => {
      const req = makeReq("denied") as Request;
      const res = makeRes() as unknown as Response;
      const next = makeNext();

      analyticsConsent(req, res, next);

      expect(next).toHaveBeenCalledTimes(1);
      expect(next).toHaveBeenCalledWith(/* no error */);
    });
  });

  describe("X-Analytics-Consent: granted (explicit)", () => {
    it("sets req.analyticsOptOut = false", () => {
      const req = makeReq("granted") as Request;
      const res = makeRes() as unknown as Response;
      const next = makeNext();

      analyticsConsent(req, res, next);

      expect(req.analyticsOptOut).toBe(false);
    });
  });

  describe("X-Analytics-Consent header absent (default)", () => {
    it("defaults to opt-in (req.analyticsOptOut = false)", () => {
      const req = makeReq(/* no header */) as Request;
      const res = makeRes() as unknown as Response;
      const next = makeNext();

      analyticsConsent(req, res, next);

      expect(req.analyticsOptOut).toBe(false);
    });

    it("still calls next()", () => {
      const req = makeReq() as Request;
      const res = makeRes() as unknown as Response;
      const next = makeNext();

      analyticsConsent(req, res, next);

      expect(next).toHaveBeenCalledTimes(1);
    });
  });

  describe("X-Analytics-Consent with arbitrary value", () => {
    it("treats any non-'denied' value as opt-in", () => {
      for (const val of ["yes", "true", "1", "GRANTED", "allow"]) {
        const req = makeReq(val) as Request;
        const res = makeRes() as unknown as Response;
        analyticsConsent(req, res, makeNext());
        expect(req.analyticsOptOut).toBe(false);
      }
    });
  });

  describe("response headers", () => {
    it("attaches X-Analytics-Opt-Out-Instructions header", () => {
      const req = makeReq() as Request;
      const res = makeRes() as unknown as Response;

      analyticsConsent(req, res, makeNext());

      expect(res.setHeader).toHaveBeenCalledWith(
        "X-Analytics-Opt-Out-Instructions",
        expect.stringContaining("X-Analytics-Consent: denied")
      );
    });

    it("includes a URL in the instructions header", () => {
      const req = makeReq() as Request;
      const res = makeRes() as unknown as Response;

      analyticsConsent(req, res, makeNext());

      const calls = (res.setHeader as jest.Mock).mock.calls as [string, string][];
      const instrValue = calls.find(([k]) => k === "X-Analytics-Opt-Out-Instructions")?.[1] ?? "";
      expect(instrValue).toMatch(/https?:\/\//);
    });
  });

  describe("next() is called exactly once regardless of header", () => {
    it.each(["denied", "granted", undefined])(
      "with header=%s calls next exactly once",
      (header) => {
        const req = makeReq(header) as Request;
        const res = makeRes() as unknown as Response;
        const next = makeNext();

        analyticsConsent(req, res, next);

        expect(next).toHaveBeenCalledTimes(1);
      }
    );
  });
});
