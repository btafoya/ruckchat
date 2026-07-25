import { test, expect, register, uniqueAccount } from "./fixtures";

test("sends a message and sees it in the channel", async ({ page }) => {
  const account = uniqueAccount("e2e-send");
  await register(page, account);

  const messageText = `Hello from e2e ${Date.now()}`;
  const composer = page.getByRole("textbox", { name: "Type a message..." });
  await composer.click();
  await composer.pressSequentially(messageText);
  await page.getByRole("button", { name: "Send" }).click();

  await expect(page.locator("article").filter({ hasText: messageText })).toBeVisible();
});
