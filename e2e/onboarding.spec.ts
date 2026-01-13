import { test, expect } from "@playwright/test";
import { OnboardingPage } from "./fixtures";

// Use ?test=onboarding to force the modal to show
const TEST_URL = "/?test=onboarding";

test.describe("Onboarding Flow", () => {
  test("should load the app successfully", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");

    const body = await page.locator("body");
    await expect(body).toBeVisible();

    const hasContent = await page.locator("body").textContent();
    expect(hasContent).toBeTruthy();
  });

  test("should show either welcome or chat list", async ({ page }) => {
    await page.goto("/");
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(1000);

    const welcomeVisible = await page
      .locator('text="Pulse for Desktop"')
      .isVisible()
      .catch(() => false);
    const chatsVisible = await page
      .locator('text="Chats"')
      .isVisible()
      .catch(() => false);

    expect(welcomeVisible || chatsVisible).toBe(true);
  });
});

test.describe("Onboarding Modal - Phone Number Input", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(TEST_URL);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
  });

  test("should display phone number input field", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.phoneInput).toBeVisible();
    await expect(onboarding.phoneInput).toHaveAttribute("type", "tel");
  });

  test("should display country selector with default country", async ({
    page,
  }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.countrySelector).toBeVisible();
    // Default is India (+91)
    const dialCode = await onboarding.getSelectedCountryDialCode();
    expect(dialCode).toBe("+91");
  });

  test("should display digit counter", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    // Default country is India (+91 = 2 digits), so counter starts at 2/15
    await expect(onboarding.digitCounter).toContainText("/15");
  });

  test("should update digit counter as user types", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("9876543210");
    // 2 (dial code) + 10 (local) = 12
    await expect(onboarding.digitCounter).toContainText("12/15");
  });

  test("should disable button when phone number is too short", async ({
    page,
  }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("123");
    expect(await onboarding.isButtonDisabled()).toBe(true);
  });

  test("should enable button when phone number is valid", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("9876543210");
    expect(await onboarding.isButtonDisabled()).toBe(false);
  });

  test("should accept valid phone number", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("9876543210");
    expect(await onboarding.isButtonDisabled()).toBe(false);
    const hasError = await onboarding.errorMessage
      .isVisible()
      .catch(() => false);
    expect(hasError).toBe(false);
  });

  test("should display current ID section", async ({ page }) => {
    const currentIdLabel = page.locator('text="Current ID"');
    await expect(currentIdLabel).toBeVisible();
  });

  test("should have autofocus on phone input", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await expect(onboarding.phoneInput).toBeFocused();
  });
});

test.describe("Country Selector", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(TEST_URL);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
  });

  test("should open dropdown when clicked", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.openCountrySelector();
    await expect(onboarding.countryDropdown).toBeVisible();
  });

  test("should show search input in dropdown", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.openCountrySelector();
    const searchInput = page.locator(
      'input[placeholder="Search country or code..."]'
    );
    await expect(searchInput).toBeVisible();
  });

  test("should filter countries by name", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.searchCountry("United States");
    const usOption = page.locator('button:has-text("United States")');
    await expect(usOption).toBeVisible();
  });

  test("should filter countries by dial code", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.searchCountry("+44");
    const ukOption = page.locator('button:has-text("United Kingdom")');
    await expect(ukOption).toBeVisible();
  });

  test("should select country and update dial code", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.selectCountry("United States");
    const dialCode = await onboarding.getSelectedCountryDialCode();
    expect(dialCode).toBe("+1");
  });

  test("should close dropdown after selection", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.selectCountry("United States");
    await expect(onboarding.countryDropdown).not.toBeVisible();
  });

  test("should show no results message for invalid search", async ({
    page,
  }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.searchCountry("xyznonexistent");
    const noResults = page.locator('text="No countries found"');
    await expect(noResults).toBeVisible();
  });

  test("should close dropdown on Escape key", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.openCountrySelector();
    await expect(onboarding.countryDropdown).toBeVisible();
    await page.keyboard.press("Escape");
    await expect(onboarding.countryDropdown).not.toBeVisible();
  });
});

test.describe("Phone Number Validation", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(TEST_URL);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
  });

  test("should show error when local number is too short", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("123");
    const errorText = await onboarding.getErrorText();
    expect(errorText).toContain("at least 4 digits");
  });

  test("should show error when local number is too long", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("1234567890123");
    const errorText = await onboarding.getErrorText();
    expect(errorText).toContain("too long");
  });

  test("should accept valid local number", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("9876543210");
    expect(await onboarding.isButtonDisabled()).toBe(false);
  });

  test("should highlight digit counter when below minimum", async ({
    page,
  }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("12");
    const counter = onboarding.digitCounter;
    await expect(counter).toHaveClass(/text-red-400/);
  });

  test("should not highlight digit counter when valid", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("9876543210");
    const counter = onboarding.digitCounter;
    const hasRedClass = await counter.getAttribute("class");
    expect(hasRedClass).not.toContain("text-red-400");
  });

  test("should only allow digits, spaces, and dashes in phone input", async ({
    page,
  }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.phoneInput.fill("987abc654xyz321");
    const value = await onboarding.phoneInput.inputValue();
    expect(value).toBe("987654321");
  });

  test("should allow formatted input with spaces", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("987 654 3210");
    expect(await onboarding.isButtonDisabled()).toBe(false);
  });

  test("should allow formatted input with dashes", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("987-654-3210");
    expect(await onboarding.isButtonDisabled()).toBe(false);
  });
});

test.describe("Country Change Updates", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(TEST_URL);
    await page.waitForLoadState("networkidle");
    await page.waitForTimeout(500);
  });

  test("should update total digit count when country changes", async ({
    page,
  }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("1234567");

    // India (+91) = 2 digits, so 2 + 7 = 9
    await expect(onboarding.digitCounter).toContainText("9/15");

    // Change to US (+1) = 1 digit, so 1 + 7 = 8
    await onboarding.selectCountry("United States");
    await expect(onboarding.digitCounter).toContainText("8/15");
  });

  test("should show full phone preview with country code", async ({ page }) => {
    const onboarding = new OnboardingPage(page);
    await onboarding.enterPhone("9876543210");

    // Should show preview like "+91 9876543210"
    const preview = page.locator('text=/\\+91.*9876543210/');
    await expect(preview).toBeVisible();
  });
});
