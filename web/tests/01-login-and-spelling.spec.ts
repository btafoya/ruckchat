import { test, expect, register, uniqueAccount } from "./fixtures";

test("registers, lands in the chat shell, and flags a misspelled word", async ({ page }) => {
  const account = uniqueAccount("e2e-login");
  await register(page, account);
  await expect(page.getByRole("heading", { name: "# general" })).toBeVisible();

  const composer = page.getByRole("textbox", { name: "Type a message..." });
  await composer.click();
  await composer.pressSequentially("helo world", { delay: 20 });

  const misspelled = page.locator("span.spell-error");
  await expect(misspelled.first()).toBeVisible({ timeout: 10_000 });

  await misspelled.first().click();
  const suggestions = page.locator("#suggestions-box");
  await expect(suggestions).toBeVisible();
  await expect(suggestions.locator("li").first()).toBeVisible();
});
