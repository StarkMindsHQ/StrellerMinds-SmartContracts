import { Request, Response } from "express";
import type { ApiResponse, ApiError } from "../types";
import { t, SupportedLocale, DEFAULT_LOCALE } from "../i18n";

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
  requestId?: string,
  locale?: SupportedLocale
): void {
  // Translate the message if a locale is provided and the code maps to a translation key
  const resolvedLocale = locale ?? DEFAULT_LOCALE;
  const translated = t(resolvedLocale, "errors", code) !== code
    ? t(resolvedLocale, "errors", code)
    : message;

  const error: ApiError = { code, message: translated };
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

/**
 * Convenience wrapper that reads locale from req automatically.
 */
export function sendLocalizedError(
  req: Request,
  res: Response,
  statusCode: number,
  code: string,
  fallbackMessage: string,
  details?: unknown,
  vars?: Record<string, string>
): void {
  const locale = req.locale ?? DEFAULT_LOCALE;
  const translated = t(locale, "errors", code, vars);
  const message = translated !== code ? translated : fallbackMessage;

  const error: ApiError = { code, message };
  if (details !== undefined) error.details = details;

  const body: ApiResponse<null> = {
    success: false,
    data: null,
    error,
    meta: {
      requestId: req.requestId ?? "unknown",
      timestamp: new Date().toISOString(),
      version: API_VERSION,
    },
  };
  res.status(statusCode).json(body);
}
