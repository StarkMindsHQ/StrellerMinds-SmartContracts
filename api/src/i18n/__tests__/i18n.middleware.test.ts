/**
 * i18n middleware integration tests
 */
import { Request, Response, NextFunction } from "express";
import { i18nMiddleware } from "../../middleware/i18n";
import { preloadLocales } from "../index";

beforeAll(() => {
  preloadLocales();
});

function makeReq(overrides: Partial<Request> = {}): Request {
  return {
    query: {},
    headers: {},
    ...overrides,
  } as unknown as Request;
}

function makeRes(): { res: Response; headers: Record<string, string> } {
  const headers: Record<string, string> = {};
  const res = {
    setHeader: (key: string, value: string) => { headers[key] = value; },
  } as unknown as Response;
  return { res, headers };
}

describe("i18nMiddleware", () => {
  it("defaults to English when no language signals present", () => {
    const req = makeReq();
    const { res, headers } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("en");
    expect(req.direction).toBe("ltr");
    expect(headers["Content-Language"]).toBe("en");
    expect(next).toHaveBeenCalled();
  });

  it("detects locale from ?lang= query param", () => {
    const req = makeReq({ query: { lang: "es" } });
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("es");
    expect(req.direction).toBe("ltr");
  });

  it("detects locale from X-Language header", () => {
    const req = makeReq({ headers: { "x-language": "fr" } });
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("fr");
  });

  it("detects locale from Accept-Language header", () => {
    const req = makeReq({ headers: { "accept-language": "zh-CN,zh;q=0.9" } });
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("zh");
  });

  it("sets RTL direction for Arabic", () => {
    const req = makeReq({ query: { lang: "ar" } });
    const { res, headers } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("ar");
    expect(req.direction).toBe("rtl");
    expect(headers["X-Text-Direction"]).toBe("rtl");
  });

  it("query param takes priority over headers", () => {
    const req = makeReq({
      query: { lang: "pt" },
      headers: { "accept-language": "fr", "x-language": "es" },
    });
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("pt");
  });

  it("X-Language header takes priority over Accept-Language", () => {
    const req = makeReq({
      headers: { "x-language": "zh", "accept-language": "fr" },
    });
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("zh");
  });

  it("falls back to English for unsupported lang query param", () => {
    const req = makeReq({ query: { lang: "de" } });
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(req.locale).toBe("en");
  });

  it("always calls next()", () => {
    const req = makeReq();
    const { res } = makeRes();
    const next = jest.fn() as NextFunction;

    i18nMiddleware(req, res, next);

    expect(next).toHaveBeenCalledTimes(1);
  });
});
