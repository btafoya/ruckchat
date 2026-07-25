import { test, expect, login, uniqueAccount, adminCredentials } from "./fixtures";

test("creates, promotes, and deletes a user from the server admin UI", async ({ page }) => {
  const admin = adminCredentials();
  test.skip(
    admin === null,
    "Requires RUCKCHAT_E2E_ADMIN_EMAIL / RUCKCHAT_E2E_ADMIN_PASSWORD for an existing " +
      "server-admin account on the target instance.",
  );

  await login(page, admin!);
  await page.goto("/admin/server/users");

  const target = uniqueAccount("e2e-admin-crud");

  await page.getByRole("button", { name: "Create User" }).click();
  await page.getByLabel("Email").fill(target.email);
  await page.getByLabel("Display name").fill(target.displayName);
  // The sidebar trigger ("Create User") and the modal's submit button
  // ("Create user") differ only in case, and Playwright's accessible-name
  // matching is case-insensitive — scope to the form to disambiguate.
  await page.locator("form").getByRole("button", { name: "Create user" }).click();
  await expect(page.getByText(/Initial password:/)).toBeVisible();
  await page.getByRole("button", { name: "Close" }).click();

  const row = page.locator("tr", { hasText: target.email });
  await expect(row).toBeVisible();

  await row.getByRole("button", { name: "Edit" }).click();
  await expect(page.getByRole("heading", { name: `Edit ${target.displayName}` })).toBeVisible();

  await page.getByRole("button", { name: "Promote to server admin" }).click();
  await expect(page.getByRole("button", { name: "Demote from server admin" })).toBeVisible();

  page.once("dialog", (dialog) => dialog.accept());
  await page.getByRole("button", { name: "Delete user permanently" }).click();

  await expect(page.locator("tr", { hasText: target.email })).toHaveCount(0);
});
