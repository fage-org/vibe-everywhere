import { expect, test } from "@playwright/test";

test.describe("browser shell", () => {
  test("boots the browser shell and can navigate into settings", async ({ page }) => {
    await page.goto("/");

    await expect(page.getByText("Vibe Runtime")).toBeVisible();
    await expect(page.getByText("Welcome to Vibe")).toBeVisible();
    await expect(page.getByText("Start a new session or continue where you left off")).toBeVisible();

    await page.getByRole("button", { name: "Settings" }).click();
    await expect(page.getByText("Save Changes")).toBeVisible();
    await expect(page.getByText("Choose your preferred language")).toBeVisible();
    await expect(page.getByText("Choose your preferred color scheme")).toBeVisible();
  });
});
