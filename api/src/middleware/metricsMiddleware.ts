import { Request, Response, NextFunction } from "express";
import { httpRequestDuration, httpRequestTotal } from "../metrics";

export function metricsMiddleware(
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const end = httpRequestDuration.startTimer({
    method: req.method,
    route: req.route?.path ?? req.path,
  });

  res.on("finish", () => {
    const labels = {
      method: req.method,
      route: req.route?.path ?? req.path,
      status_code: String(res.statusCode),
    };
    end(labels);
    httpRequestTotal.inc(labels);
  });

  next();
}
