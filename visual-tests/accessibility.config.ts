import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  testMatch: "**/accessibility.spec.ts",
  fullyParallel: false,
  retries: 0,
  timeout: 30_000,
  reporter: [
    ["list"],
    [
      "html",
      {
        outputFolder: "accessibility-report",
        open: "never",
      },
    ],
    ["json", { outputFile: "accessibility-report/results.json" }],
  ],
  use: {
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
});
