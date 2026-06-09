/**
 * Automated Accessibility Tests — WCAG 2.1 AA
 *
 * Covers:
 *  - WCAG AA compliance via axe-core
 *  - Color contrast validation
 *  - Keyboard navigation
 *  - Screen reader attributes (ARIA, roles, labels)
 *  - Remediation guidance on failure
 */
import { test, expect, Page } from "@playwright/test";
import AxeBuilder from "@axe-core/playwright";
import fs from "node:fs/promises";
import path from "node:path";

const FIXTURE_PATH = path.resolve(__dirname, "..", "site", "index.html");
const REPORT_DIR = path.resolve(__dirname, "..", "accessibility-reports");

// ── Helpers ───────────────────────────────────────────────────────────────────

async function loadPage(page: Page): Promise<void> {
  const html = await fs.readFile(FIXTURE_PATH, "utf-8");
  await page.setContent(html, { waitUntil: "load" });
  await page.setViewportSize({ width: 1280, height: 720 });
}

/** Write a JSON report with violations + remediation guidance */
async function writeReport(
  name: string,
  violations: AxeViolation[]
): Promise<void> {
  await fs.mkdir(REPORT_DIR, { recursive: true });
  const report = {
    generatedAt: new Date().toISOString(),
    totalViolations: violations.length,
    violations: violations.map((v) => ({
      id: v.id,
      impact: v.impact,
      description: v.description,
      helpUrl: v.helpUrl,
      nodes: v.nodes.length,
      remediation: buildRemediation(v),
    })),
  };
  const file = path.join(REPORT_DIR, `${name}.json`);
  await fs.writeFile(file, JSON.stringify(report, null, 2));
}

interface AxeViolation {
  id: string;
  impact: string | null;
  description: string;
  helpUrl: string;
  nodes: Array<{ html: string; failureSummary?: string }>;
}

function buildRemediation(v: AxeViolation): string {
  const remediations: Record<string, string> = {
    "color-contrast":
      "Increase text/background contrast ratio to at least 4.5:1 for normal text and 3:1 for large text (WCAG 1.4.3).",
    "image-alt":
      "Add descriptive alt attributes to all <img> elements. Use alt=\"\" for decorative images.",
    "label":
      "Associate every form input with a <label> element using for/id or aria-label/aria-labelledby.",
    "button-name":
      "Ensure all buttons have accessible names via text content, aria-label, or aria-labelledby.",
    "link-name":
      "Ensure all links have descriptive accessible names. Avoid 'click here' or 'read more'.",
    "heading-order":
      "Use headings in sequential order (h1 → h2 → h3). Do not skip heading levels.",
    "landmark-one-main":
      "Add exactly one <main> landmark to the page for screen reader navigation.",
    "region":
      "Wrap all page content in appropriate landmark regions (main, nav, header, footer).",
    "aria-required-attr":
      "Add all required ARIA attributes for the given role (e.g. aria-valuenow for progressbar).",
    "aria-valid-attr-value":
      "Ensure ARIA attribute values are valid for their type.",
    "focus-trap":
      "Ensure focus is not trapped in any component unless it is a modal dialog.",
    "skip-link":
      "Add a visible skip navigation link as the first focusable element on the page.",
  };
  return (
    remediations[v.id] ??
    `Review WCAG guidance at ${v.helpUrl} and fix the ${v.nodes.length} affected element(s).`
  );
}

// ── WCAG AA Compliance ────────────────────────────────────────────────────────

test.describe("WCAG 2.1 AA Compliance", () => {
  test("full page passes axe WCAG AA ruleset", async ({ page }, testInfo) => {
    await loadPage(page);

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa", "wcag21a", "wcag21aa"])
      .analyze();

    await writeReport(
      `wcag-aa-${testInfo.project.name}`,
      results.violations as AxeViolation[]
    );

    if (results.violations.length > 0) {
      const summary = results.violations
        .map(
          (v) =>
            `\n  [${v.impact?.toUpperCase()}] ${v.id}: ${v.description}\n` +
            `    Affects ${v.nodes.length} element(s). Fix: ${buildRemediation(v as AxeViolation)}\n` +
            `    Docs: ${v.helpUrl}`
        )
        .join("\n");
      throw new Error(
        `${results.violations.length} WCAG AA violation(s) found:${summary}`
      );
    }

    expect(results.violations).toHaveLength(0);
  });

  test("no critical or serious violations", async ({ page }, testInfo) => {
    await loadPage(page);

    const results = await new AxeBuilder({ page })
      .withTags(["wcag2a", "wcag2aa"])
      .analyze();

    const critical = results.violations.filter(
      (v) => v.impact === "critical" || v.impact === "serious"
    );

    await writeReport(
      `critical-violations-${testInfo.project.name}`,
      critical as AxeViolation[]
    );

    expect(
      critical,
      `Critical/serious violations:\n${critical.map((v) => `  ${v.id}: ${v.description}`).join("\n")}`
    ).toHaveLength(0);
  });
});

// ── Color Contrast ────────────────────────────────────────────────────────────

test.describe("Color Contrast (WCAG 1.4.3)", () => {
  test("no color-contrast violations", async ({ page }, testInfo) => {
    await loadPage(page);

    const results = await new AxeBuilder({ page })
      .withRules(["color-contrast"])
      .analyze();

    await writeReport(
      `color-contrast-${testInfo.project.name}`,
      results.violations as AxeViolation[]
    );

    expect(
      results.violations,
      results.violations
        .map(
          (v) =>
            `${v.id}: ${v.nodes.map((n) => n.html).join(", ")}\nFix: ${buildRemediation(v as AxeViolation)}`
        )
        .join("\n")
    ).toHaveLength(0);
  });

  test("muted text meets minimum contrast ratio", async ({ page }) => {
    await loadPage(page);
    // Verify the muted color (#98a7c6 on #0d1226) is present and axe passes it
    const mutedElements = page.locator(".label");
    await expect(mutedElements.first()).toBeVisible();

    const results = await new AxeBuilder({ page })
      .include(".label")
      .withRules(["color-contrast"])
      .analyze();

    expect(results.violations).toHaveLength(0);
  });
});

// ── Screen Reader Attributes ──────────────────────────────────────────────────

test.describe("Screen Reader Support", () => {
  test("page has a valid lang attribute", async ({ page }) => {
    await loadPage(page);
    const lang = await page.getAttribute("html", "lang");
    expect(lang).toBeTruthy();
    expect(lang).toMatch(/^[a-z]{2}(-[A-Z]{2})?$/);
  });

  test("page has a descriptive title", async ({ page }) => {
    await loadPage(page);
    const title = await page.title();
    expect(title.trim().length).toBeGreaterThan(0);
    expect(title).not.toBe("Untitled");
  });

  test("all images have alt attributes", async ({ page }) => {
    await loadPage(page);
    const results = await new AxeBuilder({ page })
      .withRules(["image-alt"])
      .analyze();
    expect(results.violations).toHaveLength(0);
  });

  test("landmark regions are present", async ({ page }) => {
    await loadPage(page);

    await expect(page.locator("main, [role='main']")).toHaveCount(1);
    await expect(page.locator("header, [role='banner']")).toHaveCount(1);
    await expect(page.locator("footer, [role='contentinfo']")).toHaveCount(1);
    await expect(page.locator("nav, [role='navigation']")).toHaveCount(1);
  });

  test("headings follow a logical order", async ({ page }) => {
    await loadPage(page);
    const results = await new AxeBuilder({ page })
      .withRules(["heading-order"])
      .analyze();
    expect(results.violations).toHaveLength(0);
  });

  test("progressbar has required ARIA attributes", async ({ page }) => {
    await loadPage(page);
    const bar = page.locator('[role="progressbar"]');
    await expect(bar).toHaveAttribute("aria-valuenow");
    await expect(bar).toHaveAttribute("aria-valuemin");
    await expect(bar).toHaveAttribute("aria-valuemax");
    await expect(bar).toHaveAttribute("aria-label");
  });

  test("interactive elements have accessible names", async ({ page }) => {
    await loadPage(page);
    const results = await new AxeBuilder({ page })
      .withRules(["button-name", "link-name"])
      .analyze();
    expect(results.violations).toHaveLength(0);
  });

  test("no positive tabindex values (anti-pattern)", async ({ page }) => {
    await loadPage(page);
    const badTabindex = await page
      .locator("[tabindex]")
      .evaluateAll((els) =>
        els
          .map((el) => parseInt(el.getAttribute("tabindex") ?? "0", 10))
          .filter((v) => v > 0)
      );
    expect(
      badTabindex,
      "Positive tabindex values disrupt natural tab order"
    ).toHaveLength(0);
  });

  test("skip navigation link is present and functional", async ({ page }) => {
    await loadPage(page);
    const skipLink = page.locator(".skip-link, a[href='#main-content']").first();
    await expect(skipLink).toBeAttached();

    // Tab to it and verify it becomes visible
    await page.keyboard.press("Tab");
    await expect(skipLink).toBeFocused();
  });
});

// ── Keyboard Navigation ───────────────────────────────────────────────────────

test.describe("Keyboard Navigation", () => {
  test("all interactive elements are reachable by Tab", async ({ page }) => {
    await loadPage(page);

    const focusableSelector =
      'a[href], button:not([disabled]), input:not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex="0"]';

    const focusableCount = await page.locator(focusableSelector).count();
    expect(focusableCount).toBeGreaterThan(0);

    const focused: string[] = [];
    for (let i = 0; i < focusableCount + 2; i++) {
      await page.keyboard.press("Tab");
      const tag = await page.evaluate(() => {
        const el = document.activeElement;
        return el ? `${el.tagName}#${el.id || ""}[${el.getAttribute("href") ?? el.textContent?.trim().slice(0, 20) ?? ""}]` : "none";
      });
      if (tag !== "none") focused.push(tag);
    }

    expect(focused.length).toBeGreaterThan(0);
  });

  test("focused elements have visible focus indicators", async ({ page }) => {
    await loadPage(page);

    // Tab through interactive elements and check outline is not 'none'
    const focusableSelector = 'a[href], button:not([disabled])';
    const count = await page.locator(focusableSelector).count();

    for (let i = 0; i < count; i++) {
      await page.keyboard.press("Tab");
      const outlineStyle = await page.evaluate(() => {
        const el = document.activeElement as HTMLElement | null;
        if (!el) return "none";
        const style = window.getComputedStyle(el);
        return style.outlineStyle;
      });
      // outline should not be 'none' (browsers may use box-shadow instead)
      const boxShadow = await page.evaluate(() => {
        const el = document.activeElement as HTMLElement | null;
        if (!el) return "";
        return window.getComputedStyle(el).boxShadow;
      });
      const hasFocusIndicator =
        outlineStyle !== "none" || (boxShadow !== "" && boxShadow !== "none");
      expect(
        hasFocusIndicator,
        `Element ${i + 1} has no visible focus indicator`
      ).toBe(true);
    }
  });

  test("skip link moves focus to main content", async ({ page }) => {
    await loadPage(page);

    // Focus skip link and activate it
    await page.keyboard.press("Tab");
    await page.keyboard.press("Enter");

    const focusedId = await page.evaluate(
      () => (document.activeElement as HTMLElement)?.id
    );
    expect(focusedId).toBe("main-content");
  });

  test("nav links are keyboard accessible", async ({ page }) => {
    await loadPage(page);

    const navLinks = page.locator(".nav-links a");
    const count = await navLinks.count();
    expect(count).toBeGreaterThan(0);

    // Tab past skip link, then through nav links
    await page.keyboard.press("Tab"); // skip link
    for (let i = 0; i < count; i++) {
      await page.keyboard.press("Tab");
      const focused = await page.evaluate(
        () => document.activeElement?.tagName
      );
      expect(focused).toBe("A");
    }
  });
});

// ── ARIA Roles & Attributes ───────────────────────────────────────────────────

test.describe("ARIA Roles and Attributes", () => {
  test("no invalid ARIA attributes", async ({ page }) => {
    await loadPage(page);
    const results = await new AxeBuilder({ page })
      .withRules(["aria-valid-attr", "aria-valid-attr-value", "aria-required-attr"])
      .analyze();
    expect(results.violations).toHaveLength(0);
  });

  test("no ARIA roles used on incompatible elements", async ({ page }) => {
    await loadPage(page);
    const results = await new AxeBuilder({ page })
      .withRules(["aria-allowed-role", "aria-prohibited-attr"])
      .analyze();
    expect(results.violations).toHaveLength(0);
  });

  test("live regions are correctly configured", async ({ page }) => {
    await loadPage(page);
    const liveRegions = page.locator("[aria-live]");
    const count = await liveRegions.count();
    if (count > 0) {
      for (let i = 0; i < count; i++) {
        const val = await liveRegions.nth(i).getAttribute("aria-live");
        expect(["polite", "assertive", "off"]).toContain(val);
      }
    }
  });
});
