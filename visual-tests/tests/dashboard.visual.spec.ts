import { expect, test } from "@playwright/test";
import fs from "node:fs/promises";
import path from "node:path";

const FIXTURE_PATH = path.resolve(__dirname, "..", "site", "index.html");

async function loadFixtureHtml(): Promise<string> {
  return fs.readFile(FIXTURE_PATH, "utf-8");
}

test.describe("visual regression coverage", () => {
  test("dashboard shell matches baseline", async ({ page }) => {
    await page.setContent(await loadFixtureHtml(), { waitUntil: "load" });
    await page.setViewportSize({ width: 1440, height: 900 });
    await expect(page).toHaveScreenshot("dashboard-shell.png", {
      fullPage: true,
    });
  });

  test("component cards match baseline", async ({ page }) => {
    await page.setContent(await loadFixtureHtml(), { waitUntil: "load" });
    await page.setViewportSize({ width: 1280, height: 720 });
    const cards = page.locator(".cards");
    await expect(cards).toBeVisible();
    await expect(cards).toHaveScreenshot("component-cards.png");
  });
});
