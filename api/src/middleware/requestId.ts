// @ts-nocheck
import { Request, Response, NextFunction } from "express";
import { randomUUID } from "crypto";

declare global {
  namespace Express {
    interface Request {
      requestId: string;
    }
  }
}

export function requestId(req: Request, _res: Response, next: NextFunction): void {
  req.requestId = (req.headers["x-request-id"] as string) ?? randomUUID();
  next();
}
