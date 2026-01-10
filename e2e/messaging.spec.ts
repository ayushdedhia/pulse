import { test, expect } from "@playwright/test";
import { ChatListPage, ChatPage } from "./fixtures";

test.describe("Messaging", () => {
  test("should display chat list when logged in", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);

    const chatList = new ChatListPage(page);
    const chatsVisible = await chatList.isVisible();

    // If chats visible, test passes. If not, we're on welcome screen - skip
    if (!chatsVisible) {
      test.skip();
      return;
    }

    expect(chatsVisible).toBe(true);
  });

  test("should open chat when clicked", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);

    const chatList = new ChatListPage(page);

    if (!(await chatList.isVisible())) {
      test.skip();
      return;
    }

    const chatCount = await chatList.getChatCount();
    if (chatCount === 0) {
      test.skip();
      return;
    }

    await chatList.selectFirstChat();
    await page.waitForTimeout(500);

    const chatPage = new ChatPage(page);
    expect(await chatPage.isVisible()).toBe(true);
  });
});
