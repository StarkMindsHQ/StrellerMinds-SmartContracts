import { Response } from "express";
import type { ApiResponse, ApiError } from "../types";

const API_VERSION = "1.0.0";

export function sendSuccess<T>(
  res: Response,
  data: T,
  statusCode = 200,
  requestId?: string
): void {
  const body: ApiResponse<T> = {
    success: true,
    data,
    error: null,
    meta: {
      requestId: requestId ?? "unknown",
      timestamp: new Date().toISOString(),
      version: API_VERSION,
    },
  };
  res.status(statusCode).json(body);
}

export function sendError(
  res: Response,
  statusCode: number,
  code: string,
  message: string,
  details?: unknown,
  requestId?: string
): void {
  const error: ApiError = { code, message };
  if (details !== undefined) error.details = details;

  const body: ApiResponse<null> = {
    success: false,
    data: null,
    error,
    meta: {
      requestId: requestId ?? "unknown",
      timestamp: new Date().toISOString(),
      version: API_VERSION,
    },
  };
  res.status(statusCode).json(body);
}
