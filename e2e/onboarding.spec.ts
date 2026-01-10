import { test, expect } from "@playwright/test";

test.describe("Onboarding Flow", () => {
  test("should load the app successfully", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");

    // App should load - check for any content
    const body = await page.locator("body");
    await expect(body).toBeVisible();

    // Should have either welcome screen or chat list
    const hasContent = await page.locator("body").textContent();
    expect(hasContent).toBeTruthy();
  });

  test("should show either welcome or chat list", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");

    // Wait for React to render
    await page.waitForTimeout(1000);

    // Check what screen we're on
    const welcomeVisible = await page.locator('text="Pulse for Desktop"').isVisible().catch(() => false);
    const chatsVisible = await page.locator('text="Chats"').isVisible().catch(() => false);

    // One of these should be true
    expect(welcomeVisible || chatsVisible).toBe(true);
  });
});
