import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 30000,
  retries: 0,
  workers: 1, // Run tests serially
  reporter: [["html", { open: "never" }], ["list"]],
  use: {
    baseURL: "http://localhost:1420",
    trace: "on-first-retry",
    screenshot: "only-on-failure",
  },
  // Auto-start Vite dev server
  webServer: {
    command: "npm run dev",
    url: "http://localhost:1420",
    reuseExistingServer: true,
    timeout: 30000,
  },
});
