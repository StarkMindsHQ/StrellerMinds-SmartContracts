import { Request, Response, NextFunction } from "express";
import { graphqlRateLimiter } from "../graphqlRateLimiter";

// Minimal mock helpers
function makeReq(query?: string): Partial<Request> {
  return {
    body: query !== undefined ? { query } : {},
    ip: "127.0.0.1",
    auth: undefined,
  } as Partial<Request>;
}

function makeRes(): { res: Partial<Response>; status: jest.Mock; json: jest.Mock; setHeader: jest.Mock } {
  const json = jest.fn();
  const status = jest.fn().mockReturnValue({ json });
  const setHeader = jest.fn();
  return { res: { status, json, setHeader } as unknown as Partial<Response>, status, json, setHeader };
}

describe("graphqlRateLimiter", () => {
  const next: NextFunction = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
  });

  it("passes through when no query body is present", async () => {
    const mw = graphqlRateLimiter();
    const { res } = makeRes();
    await mw(makeReq() as Request, res as Response, next);
    expect(next).toHaveBeenCalled();
  });

  it("rejects a query that exceeds depth limit", async () => {
    const mw = graphqlRateLimiter({ maxDepth: 2 });
    const { res, status } = makeRes();
    // depth 3: query { a { b { c } } }
    const query = "{ a { b { c } } }";
    await mw(makeReq(query) as Request, res as Response, next);
    expect(status).toHaveBeenCalledWith(400);
    expect(next).not.toHaveBeenCalled();
  });

  it("rejects a query that exceeds complexity limit", async () => {
    const mw = graphqlRateLimiter({ maxComplexity: 2, fieldCosts: {} });
    const { res, status } = makeRes();
    // 3 fields → complexity 3
    const query = "{ a b c }";
    await mw(makeReq(query) as Request, res as Response, next);
    expect(status).toHaveBeenCalledWith(400);
    expect(next).not.toHaveBeenCalled();
  });

  it("rejects a query that exceeds per-field occurrence limit", async () => {
    const mw = graphqlRateLimiter({ maxFieldOccurrences: 2, maxComplexity: 1000, fieldCosts: {} });
    const { res, status } = makeRes();
    // field 'id' appears 3 times via inline fragments
    const query = "{ ... on A { id } ... on B { id } ... on C { id } }";
    await mw(makeReq(query) as Request, res as Response, next);
    expect(status).toHaveBeenCalledWith(400);
    expect(next).not.toHaveBeenCalled();
  });

  it("allows a valid query within all limits", async () => {
    const mw = graphqlRateLimiter({ maxDepth: 5, maxComplexity: 50, maxFieldOccurrences: 5, fieldCosts: {} });
    const { res } = makeRes();
    const query = "{ health }";
    await mw(makeReq(query) as Request, res as Response, next);
    expect(next).toHaveBeenCalled();
  });

  it("rejects an invalid GraphQL syntax", async () => {
    const mw = graphqlRateLimiter();
    const { res, status } = makeRes();
    await mw(makeReq("{ invalid syntax !!!") as Request, res as Response, next);
    expect(status).toHaveBeenCalledWith(400);
    expect(next).not.toHaveBeenCalled();
  });

  it("enforces user-based rate limit after many requests", async () => {
    const mw = graphqlRateLimiter({ requestsPerMinute: 2, burstLimit: 1, maxComplexity: 1000, fieldCosts: {} });
    const req = makeReq("{ health }") as Request;
    // First two requests should pass (main limit)
    for (let i = 0; i < 2; i++) {
      const { res } = makeRes();
      await mw(req, res as Response, next);
    }
    // Third request uses burst
    const { res: res3 } = makeRes();
    await mw(req, res3 as Response, next);
    // Fourth request should be rate limited
    const { res: res4, status: status4 } = makeRes();
    await mw(req, res4 as Response, next);
    expect(status4).toHaveBeenCalledWith(429);
  });
});
