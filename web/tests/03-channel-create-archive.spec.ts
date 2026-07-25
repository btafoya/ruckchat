import { test, expect, register, uniqueAccount } from "./fixtures";

test("creates a channel then archives it", async ({ page }) => {
  const account = uniqueAccount("e2e-channel");
  await register(page, account);

  const channelName = `e2e-channel-${Date.now()}`;

  // The sidebar "+" trigger and the modal's submit button share the same
  // accessible name ("Create channel"); the trigger is unambiguous here
  // because the modal doesn't exist yet.
  await page.getByRole("button", { name: "Create channel" }).click();
  await page.getByLabel("Name").fill(channelName);
  await page.locator("form").getByRole("button", { name: "Create channel" }).click();

  await expect(page).toHaveURL(/\/channel\/[0-9a-f-]+$/);
  await expect(page.getByRole("link", { name: `# ${channelName}` })).toBeVisible();

  await page.getByRole("button", { name: `Channel settings for ${channelName}` }).click();
  await page.getByRole("button", { name: "Archive channel" }).click();

  // Archived channels move into a collapsed <details> section; active
  // channels are links, archived ones are plain buttons, so the name alone
  // disambiguates which list the channel is in.
  await page.getByText("Archived", { exact: true }).click();
  await expect(page.getByRole("button", { name: `# ${channelName}` })).toBeVisible();
  await expect(page.getByRole("link", { name: `# ${channelName}` })).toHaveCount(0);
});
