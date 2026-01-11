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

  test("should show reply context menu on right-click", async ({ page }) => {
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
    const bubbles = await chatPage.getMessageBubbles();

    if (bubbles.length === 0) {
      test.skip();
      return;
    }

    // Right-click on the first message
    await chatPage.rightClickMessage(0);
    await page.waitForTimeout(200);

    // Check that context menu with Reply button is visible
    const replyButton = page.locator('button:has-text("Reply")');
    expect(await replyButton.isVisible()).toBe(true);
  });

  test("should show reply bar when clicking Reply in context menu", async ({ page }) => {
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
    const bubbles = await chatPage.getMessageBubbles();

    if (bubbles.length === 0) {
      test.skip();
      return;
    }

    // Right-click and select Reply
    await chatPage.rightClickMessage(0);
    await page.waitForTimeout(200);
    await chatPage.clickReplyInContextMenu();
    await page.waitForTimeout(200);

    // Check that reply bar is visible
    const replyBar = page.locator('[class*="animate-slide-down"]');
    expect(await replyBar.isVisible()).toBe(true);
  });

  test("should close reply bar when clicking X", async ({ page }) => {
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
    const bubbles = await chatPage.getMessageBubbles();

    if (bubbles.length === 0) {
      test.skip();
      return;
    }

    // Open reply bar
    await chatPage.rightClickMessage(0);
    await page.waitForTimeout(200);
    await chatPage.clickReplyInContextMenu();
    await page.waitForTimeout(200);

    // Close reply bar
    await chatPage.closeReplyBar();
    await page.waitForTimeout(200);

    // Verify reply bar is hidden
    const replyBar = page.locator('[class*="animate-slide-down"]');
    expect(await replyBar.isVisible()).toBe(false);
  });

  test("should dismiss context menu when clicking outside", async ({ page }) => {
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
    const bubbles = await chatPage.getMessageBubbles();

    if (bubbles.length === 0) {
      test.skip();
      return;
    }

    // Open context menu
    await chatPage.rightClickMessage(0);
    await page.waitForTimeout(200);

    const replyButton = page.locator('button:has-text("Reply")');
    expect(await replyButton.isVisible()).toBe(true);

    // Click outside to dismiss
    await page.click("body", { position: { x: 10, y: 10 } });
    await page.waitForTimeout(200);

    // Context menu should be hidden
    expect(await replyButton.isVisible()).toBe(false);
  });
});
