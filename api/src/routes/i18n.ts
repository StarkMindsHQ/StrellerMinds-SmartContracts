/**
 * GET  /api/v1/i18n/languages  — list supported languages
 * GET  /api/v1/i18n/locale     — get current locale info for the request
 */
import { Router, Request, Response } from "express";
import {
  SUPPORTED_LOCALES,
  getLocaleData,
  getDirection,
  formatDate,
  formatNumber,
  formatCurrency,
} from "../i18n";

const router = Router();

// List all supported languages
router.get("/languages", (_req: Request, res: Response) => {
  const languages = SUPPORTED_LOCALES.map((locale) => {
    const data = getLocaleData(locale);
    return {
      locale,
      language: data.meta.language,
      direction: data.meta.direction,
      dateFormat: data.meta.dateFormat,
    };
  });
  res.json({ success: true, data: { languages } });
});

// Get current locale info + formatting examples
router.get("/locale", (req: Request, res: Response) => {
  const locale = req.locale;
  const data = getLocaleData(locale);
  const now = Math.floor(Date.now() / 1000);

  res.json({
    success: true,
    data: {
      locale,
      language: data.meta.language,
      direction: getDirection(locale),
      dateFormat: data.meta.dateFormat,
      timeFormat: data.meta.timeFormat,
      numberFormat: data.meta.numberFormat,
      examples: {
        date: formatDate(locale, now, "long"),
        number: formatNumber(locale, 1234567.89),
        currency: formatCurrency(locale, 1234.56),
      },
    },
  });
});

export default router;
