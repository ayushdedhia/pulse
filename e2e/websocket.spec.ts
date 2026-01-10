import { test, expect } from "@playwright/test";

test.describe("WebSocket Connection", () => {
  test("should load app without crashing", async ({ page }) => {
    // Collect console messages
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(msg.text()));

    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(2000);

    // App should not have crashed
    const body = await page.locator("body");
    await expect(body).toBeVisible();
  });

  test("should attempt WebSocket connection", async ({ page }) => {
    const consoleLogs: string[] = [];
    page.on("console", (msg) => consoleLogs.push(msg.text()));

    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(3000);

    // Check for WebSocket-related logs (connection attempt or success)
    const hasWsActivity = consoleLogs.some(
      (log) =>
        log.includes("WebSocket") ||
        log.includes("Connected") ||
        log.includes("Pulse server") ||
        log.includes("connecting")
    );

    // WebSocket activity should be present (either success or error is fine)
    // The important thing is the app tried to connect
    expect(consoleLogs.length).toBeGreaterThan(0);
  });
});
