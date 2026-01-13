import { test as base, type Page } from "@playwright/test";

// Extend the base test with custom fixtures
export const test = base.extend<{
  appPage: Page;
}>({
  appPage: async ({ page }, use) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await use(page);
  },
});

export { expect } from "@playwright/test";

// Page object helpers
export class OnboardingPage {
  constructor(private page: Page) {}

  async isVisible() {
    return this.page
      .locator('text="Enter your phone number"')
      .isVisible()
      .catch(() => false);
  }

  async waitForLoad() {
    await this.page.waitForSelector('text="Enter your phone number"', {
      timeout: 10000,
    });
  }

  get countrySelector() {
    // Button with country code badge (e.g., "IN +91") or "Select" text
    return this.page.locator('button:has-text("+")').first();
  }

  get countryDropdown() {
    return this.page.locator('input[placeholder="Search country or code..."]');
  }

  get phoneInput() {
    return this.page.locator('input[type="tel"]');
  }

  get continueButton() {
    return this.page.locator('button:has-text("Continue with this number")');
  }

  get savingButton() {
    return this.page.locator('button:has-text("Saving...")');
  }

  get errorMessage() {
    return this.page.locator(".text-red-500");
  }

  get digitCounter() {
    return this.page.locator('p.tabular-nums');
  }

  async openCountrySelector() {
    await this.countrySelector.click();
  }

  async selectCountry(name: string) {
    await this.openCountrySelector();
    await this.countryDropdown.fill(name);
    await this.page.locator(`button:has-text("${name}")`).first().click();
  }

  async searchCountry(query: string) {
    await this.openCountrySelector();
    await this.countryDropdown.fill(query);
  }

  async enterPhone(phone: string) {
    await this.phoneInput.fill(phone);
  }

  async submit() {
    await this.continueButton.click();
  }

  async getErrorText() {
    return this.errorMessage.textContent();
  }

  async isButtonDisabled() {
    return this.continueButton.isDisabled();
  }

  async getSelectedCountryDialCode() {
    const text = await this.countrySelector.textContent();
    const match = text?.match(/\+\d+/);
    return match ? match[0] : null;
  }
}

export class ChatListPage {
  constructor(private page: Page) {}

  async isVisible() {
    return this.page.locator('text="Chats"').isVisible().catch(() => false);
  }

  async waitForLoad() {
    await this.page.waitForSelector('text="Chats"', { timeout: 10000 });
  }

  async getChatCount() {
    return this.page.locator('[data-testid="chat-item"]').count();
  }

  async selectFirstChat() {
    await this.page.locator('[data-testid="chat-item"]').first().click();
  }
}

export class ChatPage {
  constructor(private page: Page) {}

  async isVisible() {
    const input = this.page.locator('[data-testid="message-input"]');
    return input.isVisible().catch(() => false);
  }

  async sendMessage(content: string) {
    await this.page.fill('[data-testid="message-input"]', content);
    await this.page.keyboard.press("Enter");
  }

  async waitForMessage(content: string, timeout = 5000) {
    await this.page.waitForSelector(`text="${content}"`, { timeout });
  }

  async getMessageBubbles() {
    return this.page.locator('[class*="bubble-tail"]').all();
  }

  async rightClickMessage(index: number) {
    const bubbles = await this.getMessageBubbles();
    if (bubbles.length > index) {
      await bubbles[index].click({ button: "right" });
    }
  }

  async clickReplyInContextMenu() {
    await this.page.locator('button:has-text("Reply")').click();
  }

  async isReplyBarVisible() {
    return this.page.locator('text="Replying to"').isVisible().catch(() => false) ||
           this.page.locator('[class*="animate-slide-down"]').isVisible().catch(() => false);
  }

  async closeReplyBar() {
    await this.page.locator('[class*="animate-slide-down"] button').click();
  }
}
