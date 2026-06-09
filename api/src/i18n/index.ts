/**
 * i18n - Internationalization module
 *
 * Supports: en, es, fr, ar, zh, pt
 * Features: language detection, translation, RTL, date/number formatting
 */
import path from "path";
import fs from "fs";

export type SupportedLocale = "en" | "es" | "fr" | "ar" | "zh" | "pt";

export interface LocaleMeta {
  language: string;
  locale: SupportedLocale;
  direction: "ltr" | "rtl";
  dateFormat: string;
  timeFormat: string;
  numberFormat: {
    decimal: string;
    thousands: string;
    currency: string;
  };
}

export interface LocaleData {
  meta: LocaleMeta;
  errors: Record<string, string>;
  health: Record<string, string>;
  certificate: Record<string, string>;
  formats: Record<string, string>;
}

export const SUPPORTED_LOCALES: SupportedLocale[] = ["en", "es", "fr", "ar", "zh", "pt"];
export const DEFAULT_LOCALE: SupportedLocale = "en";

// RTL languages
export const RTL_LOCALES: SupportedLocale[] = ["ar"];

// Cache loaded locale data in memory (loaded once at startup)
const localeCache = new Map<SupportedLocale, LocaleData>();

function loadLocale(locale: SupportedLocale): LocaleData {
  if (localeCache.has(locale)) {
    return localeCache.get(locale)!;
  }
  const filePath = path.join(__dirname, "locales", `${locale}.json`);
  const raw = fs.readFileSync(filePath, "utf-8");
  const data = JSON.parse(raw) as LocaleData;
  localeCache.set(locale, data);
  return data;
}

/** Pre-load all locales at startup to avoid runtime I/O */
export function preloadLocales(): void {
  for (const locale of SUPPORTED_LOCALES) {
    loadLocale(locale);
  }
}

/** Get locale data (from cache) */
export function getLocaleData(locale: SupportedLocale): LocaleData {
  return loadLocale(locale);
}

/**
 * Detect locale from Accept-Language header.
 * Falls back to DEFAULT_LOCALE if no match found.
 */
export function detectLocale(acceptLanguage?: string): SupportedLocale {
  if (!acceptLanguage) return DEFAULT_LOCALE;

  // Parse "Accept-Language: en-US,en;q=0.9,es;q=0.8"
  const candidates = acceptLanguage
    .split(",")
    .map((part) => {
      const [tag, q] = part.trim().split(";q=");
      return { tag: tag.trim().toLowerCase(), q: q ? parseFloat(q) : 1.0 };
    })
    .sort((a, b) => b.q - a.q);

  for (const { tag } of candidates) {
    // Exact match first (e.g. "es")
    if (SUPPORTED_LOCALES.includes(tag as SupportedLocale)) {
      return tag as SupportedLocale;
    }
    // Language prefix match (e.g. "es-MX" → "es")
    const prefix = tag.split("-")[0] as SupportedLocale;
    if (SUPPORTED_LOCALES.includes(prefix)) {
      return prefix;
    }
  }

  return DEFAULT_LOCALE;
}

/**
 * Interpolate template variables: "Hello {{name}}" + {name: "World"} → "Hello World"
 */
function interpolate(template: string, vars?: Record<string, string>): string {
  if (!vars) return template;
  return template.replace(/\{\{(\w+)\}\}/g, (_, key) => vars[key] ?? `{{${key}}}`);
}

/**
 * Translate a key in the given namespace.
 * Falls back to English if key not found in target locale.
 */
export function t(
  locale: SupportedLocale,
  namespace: keyof Omit<LocaleData, "meta" | "formats">,
  key: string,
  vars?: Record<string, string>
): string {
  const data = getLocaleData(locale);
  const ns = data[namespace] as Record<string, string>;
  const template = ns[key];

  if (template) return interpolate(template, vars);

  // Fallback to English
  if (locale !== DEFAULT_LOCALE) {
    const fallback = getLocaleData(DEFAULT_LOCALE);
    const fallbackNs = fallback[namespace] as Record<string, string>;
    const fallbackTemplate = fallbackNs[key];
    if (fallbackTemplate) return interpolate(fallbackTemplate, vars);
  }

  return key; // Last resort: return the key itself
}

/** Format a Unix timestamp as a localized date string */
export function formatDate(
  locale: SupportedLocale,
  timestamp: number,
  style: "short" | "long" = "short"
): string {
  const date = new Date(timestamp * 1000);
  try {
    const intlLocale = locale === "zh" ? "zh-CN" : locale === "pt" ? "pt-BR" : locale;
    return new Intl.DateTimeFormat(intlLocale, {
      dateStyle: style === "long" ? "long" : "short",
    }).format(date);
  } catch {
    // Fallback manual format
    const y = date.getFullYear();
    const m = String(date.getMonth() + 1).padStart(2, "0");
    const d = String(date.getDate()).padStart(2, "0");
    return `${y}-${m}-${d}`;
  }
}

/** Format a number according to locale conventions */
export function formatNumber(locale: SupportedLocale, value: number): string {
  try {
    const intlLocale = locale === "zh" ? "zh-CN" : locale === "pt" ? "pt-BR" : locale;
    return new Intl.NumberFormat(intlLocale).format(value);
  } catch {
    return String(value);
  }
}

/** Format a currency value according to locale conventions */
export function formatCurrency(
  locale: SupportedLocale,
  value: number,
  currencyCode?: string
): string {
  const meta = getLocaleData(locale).meta;
  const currency = currencyCode ?? meta.numberFormat.currency;
  try {
    const intlLocale = locale === "zh" ? "zh-CN" : locale === "pt" ? "pt-BR" : locale;
    return new Intl.NumberFormat(intlLocale, {
      style: "currency",
      currency,
    }).format(value);
  } catch {
    return `${currency} ${value}`;
  }
}

/** Check if a locale is RTL */
export function isRTL(locale: SupportedLocale): boolean {
  return RTL_LOCALES.includes(locale);
}

/** Get text direction for a locale */
export function getDirection(locale: SupportedLocale): "ltr" | "rtl" {
  return isRTL(locale) ? "rtl" : "ltr";
}
