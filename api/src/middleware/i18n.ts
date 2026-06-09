/**
 * i18n middleware
 *
 * Detects locale from (in priority order):
 *   1. ?lang= query param
 *   2. X-Language header
 *   3. Accept-Language header
 *
 * Attaches locale info to req and sets Content-Language response header.
 */
import { Request, Response, NextFunction } from "express";
import {
  detectLocale,
  getLocaleData,
  getDirection,
  SupportedLocale,
  SUPPORTED_LOCALES,
  DEFAULT_LOCALE,
} from "../i18n";

declare global {
  namespace Express {
    interface Request {
      locale: SupportedLocale;
      direction: "ltr" | "rtl";
    }
  }
}

export function i18nMiddleware(req: Request, res: Response, next: NextFunction): void {
  let locale: SupportedLocale = DEFAULT_LOCALE;

  // 1. Query param: ?lang=es
  const queryLang = req.query.lang as string | undefined;
  if (queryLang && SUPPORTED_LOCALES.includes(queryLang.toLowerCase() as SupportedLocale)) {
    locale = queryLang.toLowerCase() as SupportedLocale;
  }
  // 2. X-Language header
  else if (req.headers["x-language"]) {
    const headerLang = (req.headers["x-language"] as string).toLowerCase();
    if (SUPPORTED_LOCALES.includes(headerLang as SupportedLocale)) {
      locale = headerLang as SupportedLocale;
    }
  }
  // 3. Accept-Language header
  else {
    locale = detectLocale(req.headers["accept-language"]);
  }

  req.locale = locale;
  req.direction = getDirection(locale);

  // Set response headers
  res.setHeader("Content-Language", locale);
  if (req.direction === "rtl") {
    res.setHeader("X-Text-Direction", "rtl");
  }

  next();
}
