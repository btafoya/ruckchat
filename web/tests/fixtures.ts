import { test, expect, type Page } from "@playwright/test";

export { test, expect };

export interface TestAccount {
  email: string;
  displayName: string;
  password: string;
  orgName: string;
  orgSlug: string;
}

let uniqueCounter = 0;

/** Generates a collision-free account so specs never touch real user data. */
export function uniqueAccount(prefix: string): TestAccount {
  uniqueCounter += 1;
  const id = `${Date.now()}-${uniqueCounter}`;
  return {
    email: `${prefix}-${id}@example.com`,
    displayName: `${prefix} ${id}`,
    password: "correct horse battery staple",
    orgName: `${prefix} Org ${id}`,
    orgSlug: `${prefix}-org-${id}`,
  };
}

export async function register(page: Page, account: TestAccount): Promise<void> {
  await page.goto("/");
  await page.getByRole("button", { name: "Don't have an account? Create one" }).click();
  await page.getByLabel("Email").fill(account.email);
  await page.getByLabel("Password").fill(account.password);
  await page.getByLabel("Display name").fill(account.displayName);
  await page.getByLabel("Organization name").fill(account.orgName);
  await page.getByLabel("Organization slug").fill(account.orgSlug);
  await page.getByRole("button", { name: "Create account" }).click();
  await page.waitForURL(/\/org\/.+\/channel\/.+/);
}

export async function login(
  page: Page,
  account: Pick<TestAccount, "email" | "password">,
): Promise<void> {
  await page.goto("/");
  await page.getByLabel("Email").fill(account.email);
  await page.getByLabel("Password").fill(account.password);
  await page.getByRole("button", { name: "Sign in" }).click();
  await page.waitForURL(/\/org\/.+\/channel\/.+/);
}

/**
 * The admin-CRUD spec needs an account that's already a server admin — on a
 * live, already-populated instance there's no "first user" to auto-promote.
 * Supplied via env vars rather than guessed/hardcoded; the spec skips itself
 * when they're absent instead of failing.
 */
export function adminCredentials(): Pick<TestAccount, "email" | "password"> | null {
  const email = process.env.RUCKCHAT_E2E_ADMIN_EMAIL;
  const password = process.env.RUCKCHAT_E2E_ADMIN_PASSWORD;
  if (!email || !password) {
    return null;
  }
  return { email, password };
}
