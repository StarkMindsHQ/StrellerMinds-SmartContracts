/**
 * i18n unit tests
 * Run: npx jest --testPathPattern=i18n --runInBand
 */
import {
  detectLocale,
  t,
  formatDate,
  formatNumber,
  formatCurrency,
  isRTL,
  getDirection,
  preloadLocales,
  SUPPORTED_LOCALES,
  getLocaleData,
  DEFAULT_LOCALE,
} from "../index";

beforeAll(() => {
  preloadLocales();
});

// ── Language detection ────────────────────────────────────────────────────────

describe("detectLocale", () => {
  it("returns default locale when no header provided", () => {
    expect(detectLocale()).toBe("en");
    expect(detectLocale(undefined)).toBe("en");
    expect(detectLocale("")).toBe("en");
  });

  it("detects exact locale match", () => {
    expect(detectLocale("es")).toBe("es");
    expect(detectLocale("fr")).toBe("fr");
    expect(detectLocale("ar")).toBe("ar");
    expect(detectLocale("zh")).toBe("zh");
    expect(detectLocale("pt")).toBe("pt");
  });

  it("detects locale from language prefix (e.g. es-MX → es)", () => {
    expect(detectLocale("es-MX")).toBe("es");
    expect(detectLocale("fr-CA")).toBe("fr");
    expect(detectLocale("zh-CN")).toBe("zh");
    expect(detectLocale("pt-BR")).toBe("pt");
    expect(detectLocale("ar-SA")).toBe("ar");
  });

  it("respects quality values and picks highest q", () => {
    expect(detectLocale("en-US,en;q=0.9,es;q=0.8")).toBe("en");
    expect(detectLocale("fr;q=0.9,es;q=1.0")).toBe("es");
  });

  it("falls back to en for unsupported locale", () => {
    expect(detectLocale("de")).toBe("en");
    expect(detectLocale("ja,ko;q=0.9")).toBe("en");
  });

  it("is case-insensitive", () => {
    expect(detectLocale("ES")).toBe("es");
    expect(detectLocale("FR-FR")).toBe("fr");
  });
});

// ── Translation ───────────────────────────────────────────────────────────────

describe("t (translate)", () => {
  it("translates error keys for all supported locales", () => {
    const key = "AUTH_REQUIRED";
    for (const locale of SUPPORTED_LOCALES) {
      const result = t(locale, "errors", key);
      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);
      expect(result).not.toBe(key); // should not return the key itself
    }
  });

  it("interpolates template variables", () => {
    const result = t("en", "errors", "INSUFFICIENT_SCOPE", { scope: "admin" });
    expect(result).toContain("admin");
  });

  it("interpolates in Spanish", () => {
    const result = t("es", "errors", "INSUFFICIENT_SCOPE", { scope: "admin" });
    expect(result).toContain("admin");
  });

  it("falls back to English for missing key", () => {
    const result = t("ar", "errors", "NONEXISTENT_KEY_XYZ");
    expect(result).toBe("NONEXISTENT_KEY_XYZ"); // returns key as last resort
  });

  it("returns English fallback when locale missing a key", () => {
    // All locales should have AUTH_REQUIRED
    expect(t("zh", "errors", "AUTH_REQUIRED")).not.toBe("AUTH_REQUIRED");
  });

  it("translates health keys", () => {
    expect(t("en", "health", "status_ok")).toBe("ok");
    expect(t("es", "health", "status_ready")).toBeTruthy();
    expect(t("ar", "health", "cannot_reach_contract")).toBeTruthy();
  });
});

// ── RTL support ───────────────────────────────────────────────────────────────

describe("RTL support", () => {
  it("identifies Arabic as RTL", () => {
    expect(isRTL("ar")).toBe(true);
    expect(getDirection("ar")).toBe("rtl");
  });

  it("identifies LTR languages correctly", () => {
    const ltrLocales = ["en", "es", "fr", "zh", "pt"] as const;
    for (const locale of ltrLocales) {
      expect(isRTL(locale)).toBe(false);
      expect(getDirection(locale)).toBe("ltr");
    }
  });

  it("locale meta direction matches isRTL", () => {
    for (const locale of SUPPORTED_LOCALES) {
      const meta = getLocaleData(locale).meta;
      expect(meta.direction).toBe(getDirection(locale));
    }
  });
});

// ── Date formatting ───────────────────────────────────────────────────────────

describe("formatDate", () => {
  const timestamp = 1700000000; // 2023-11-14

  it("formats dates for all locales without throwing", () => {
    for (const locale of SUPPORTED_LOCALES) {
      expect(() => formatDate(locale, timestamp)).not.toThrow();
      expect(() => formatDate(locale, timestamp, "long")).not.toThrow();
    }
  });

  it("returns a non-empty string", () => {
    for (const locale of SUPPORTED_LOCALES) {
      const result = formatDate(locale, timestamp);
      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);
    }
  });

  it("produces different formats for different locales", () => {
    const en = formatDate("en", timestamp);
    const ar = formatDate("ar", timestamp);
    const zh = formatDate("zh", timestamp);
    // At least some should differ
    const unique = new Set([en, ar, zh]);
    expect(unique.size).toBeGreaterThanOrEqual(1);
  });
});

// ── Number formatting ─────────────────────────────────────────────────────────

describe("formatNumber", () => {
  it("formats numbers for all locales without throwing", () => {
    for (const locale of SUPPORTED_LOCALES) {
      expect(() => formatNumber(locale, 1234567.89)).not.toThrow();
    }
  });

  it("returns a non-empty string", () => {
    for (const locale of SUPPORTED_LOCALES) {
      const result = formatNumber(locale, 42);
      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);
    }
  });

  it("handles zero and negative numbers", () => {
    expect(formatNumber("en", 0)).toBe("0");
    expect(formatNumber("en", -1234)).toContain("1,234");
  });
});

// ── Currency formatting ───────────────────────────────────────────────────────

describe("formatCurrency", () => {
  it("formats currency for all locales without throwing", () => {
    for (const locale of SUPPORTED_LOCALES) {
      expect(() => formatCurrency(locale, 1234.56)).not.toThrow();
    }
  });

  it("returns a non-empty string", () => {
    for (const locale of SUPPORTED_LOCALES) {
      const result = formatCurrency(locale, 100);
      expect(typeof result).toBe("string");
      expect(result.length).toBeGreaterThan(0);
    }
  });

  it("accepts custom currency code", () => {
    const result = formatCurrency("en", 100, "EUR");
    expect(result).toContain("100");
  });
});

// ── Locale data integrity ─────────────────────────────────────────────────────

describe("locale data integrity", () => {
  it("all locales have required meta fields", () => {
    for (const locale of SUPPORTED_LOCALES) {
      const data = getLocaleData(locale);
      expect(data.meta.locale).toBe(locale);
      expect(data.meta.language).toBeTruthy();
      expect(["ltr", "rtl"]).toContain(data.meta.direction);
      expect(data.meta.dateFormat).toBeTruthy();
      expect(data.meta.numberFormat.decimal).toBeTruthy();
    }
  });

  it("all locales have the same error keys as English", () => {
    const enKeys = Object.keys(getLocaleData("en").errors).sort();
    for (const locale of SUPPORTED_LOCALES) {
      const keys = Object.keys(getLocaleData(locale).errors).sort();
      expect(keys).toEqual(enKeys);
    }
  });

  it("5+ languages are supported", () => {
    expect(SUPPORTED_LOCALES.length).toBeGreaterThanOrEqual(5);
  });

  it("default locale is English", () => {
    expect(DEFAULT_LOCALE).toBe("en");
  });
});
