import { Locator, Page, expect } from '@playwright/test';

export class Dialogs {
  page: Page;

  constructor(page: Page) {
    this.page = page;
  }

  dialogLocator(): Locator {
    return this.page.getByRole('dialog');
  }

  async waitForVisible() {
    await expect(this.dialogLocator()).toBeVisible();
  }

  async waitForHidden() {
    await expect(this.dialogLocator()).not.toBeVisible();
  }

  async waitForTitle(title: string | RegExp) {
    await expect(
      this.dialogLocator().getByRole('heading', { name: title })
    ).toBeVisible();
  }

  async waitForMessage(message: string | RegExp) {
    await expect(this.dialogLocator().getByText(message)).toBeVisible();
  }

  async waitForButton(text: string) {
    await expect(
      this.dialogLocator().getByRole('button', { name: text })
    ).toBeVisible();
  }

  async clickButton(text: string) {
    await this.dialogLocator().getByRole('button', { name: text }).click();
  }

  async clickButtonWait(text: string) {
    await this.clickButton(text);
    await this.waitForHidden();
  }

  async waitForDialog(
    title: string | RegExp,
    message: string | RegExp,
    primary: string,
    secondary?: string
  ) {
    await this.waitForVisible();
    await this.waitForTitle(title);
    await this.waitForMessage(message);
    await this.waitForButton(primary);

    if (secondary !== undefined) {
      await this.waitForButton(secondary);
    }
  }
}
