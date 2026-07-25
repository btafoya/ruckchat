import { defineConfig } from "@playwright/test";

// Web UI e2e tests run against the live deployed instance, not a locally
// spun-up server/database (see CLAUDE.md's "Web UI e2e testing" note).
// Each spec self-registers a fresh, uniquely-named account and organization,
// so tests are isolated from real user data and safe to re-run. `workers: 1`
// keeps load on the shared server gentle rather than firing tests
// concurrently.
const BASE_URL = process.env.RUCKCHAT_E2E_BASE_URL ?? "https://ruck.premadev.com";

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  workers: 1,
  retries: 0,
  reporter: "list",
  use: {
    baseURL: BASE_URL,
    trace: "retain-on-failure",
  },
});
