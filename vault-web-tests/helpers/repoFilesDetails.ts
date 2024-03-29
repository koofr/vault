import { Locator, Page, expect } from '@playwright/test';

import { Repo } from '../vault-wasm-nodejs/vault-wasm';

import { test } from './base';
import {
  breadcrumbsLocator,
  navbarCloseLocator,
  navbarLocator,
  navbarLogoLocator,
} from './dashboard';
import { Dialogs } from './dialogs';
import { joinParentName } from './pathUtils';
import {
  filesTableClickFile,
  filesTableFileSelected,
  filesTableRowLocator,
  filesTableRowNameLocator,
  filesTableSelectFile,
  openFileBrowser,
  toolbarEditTextFile,
} from './repoFiles';
import { createRepo, unlockRepo } from './repos';
import { sleep } from './time';
import { WebVaultClient } from './webVaultClient';

export function fileDetailsNameLocator(page: Page): Locator {
  return navbarLocator(page).getByLabel('File name');
}

export function fileDetailsErrorLocator(page: Page): Locator {
  return navbarLocator(page).getByLabel('File error');
}

export function pdfViewerLocator(page: Page): Locator {
  return page.getByTitle('PDF viewer');
}

export async function pdfViewerBodyLocator(page: Page): Promise<Locator> {
  const pdfViewerId = await pdfViewerLocator(page).getAttribute('id');

  return page.frameLocator(`#${pdfViewerId}`).locator('body');
}

export class TextEditor {
  page: Page;
  webVaultClient: WebVaultClient;
  repo: Repo;
  dialogs: Dialogs;

  unlocked: boolean;
  currentEncryptedParentPath: string;
  currentName: string;
  currentEncryptedName: string;
  currentEncryptedPath: string;
  currentContent: string;
  serverContent: string;
  autosaveMs?: number;

  constructor(
    page: Page,
    webVaultClient: WebVaultClient,
    repo: Repo,
    dialogs: Dialogs,
  ) {
    this.page = page;
    this.webVaultClient = webVaultClient;
    this.repo = repo;
    this.dialogs = dialogs;

    this.unlocked = false;
    this.currentEncryptedParentPath = '';
    this.currentName = '';
    this.currentEncryptedName = '';
    this.currentEncryptedPath = '';
    this.currentContent = '';
    this.serverContent = '';
  }

  async openApp() {
    await this.page.goto('/');
    await this.expectFilesRootOpen();
  }

  encryptName(name: string): string {
    return this.webVaultClient.encryptFilename(this.repo.id, name);
  }

  setCurrentName(name: string) {
    this.currentName = name;
    this.currentEncryptedName = this.encryptName(name);
    this.currentEncryptedPath = joinParentName(
      this.currentEncryptedParentPath,
      this.currentEncryptedName,
    );
  }

  async createFile() {
    this.serverContent = 'editorcontent';
    this.currentContent = this.serverContent;
    this.currentEncryptedParentPath = '/';
    this.setCurrentName('file.txt');

    await this.webVaultClient.webVault.transfersUpload(
      this.repo.id,
      this.currentEncryptedParentPath,
      this.currentName,
      new Blob([this.currentContent]),
    );
  }

  async unlockRepo() {
    if (!this.unlocked) {
      await unlockRepo(this.page);

      this.unlocked = true;
    }
  }

  async openParentFolder() {
    await openFileBrowser(
      this.page,
      this.repo.id,
      this.currentEncryptedParentPath,
    );
    await this.unlockRepo();
  }

  editorUrl(encryptedPath: string, isEditing: boolean) {
    let url = `/repos/${this.repo.id}/details?path=${encodeURIComponent(
      encryptedPath,
    )}`;

    if (isEditing) {
      url += '&editing=true';
    }

    if (this.autosaveMs !== undefined) {
      url += `&autosave=${this.autosaveMs}`;
    }

    return url;
  }

  async editorNavigate(encryptedPath: string, editing: boolean) {
    await this.page.goto(this.editorUrl(encryptedPath, editing));
    await this.unlockRepo();
  }

  autorenamed() {
    this.setCurrentName('file (1).txt');
    this.currentContent = 'editorcontent1';
  }

  async expectHeaderName(name: string) {
    await expect(fileDetailsNameLocator(this.page)).toHaveText(name);
  }

  async expectHeaderNameMatch() {
    await expect(fileDetailsNameLocator(this.page)).toHaveText(
      this.currentName,
    );
  }

  async expectHeaderError(error: string) {
    await expect(fileDetailsErrorLocator(this.page)).toHaveText(error);
  }

  async expectHeaderErrorStays(error: string, durationMs = 500, sleepMs = 100) {
    const deadline = Date.now() + durationMs;

    while (Date.now() < deadline) {
      await this.expectHeaderError(error);
      await sleep(sleepMs);
    }
  }

  async expectEmptyNavbarNav() {
    await expect(navbarLocator(this.page).getByLabel('Navbar nav')).toBeEmpty();
  }

  async expectURLPathMatch() {
    await this.page.waitForURL(
      (url) => url.searchParams.get('path') === this.currentEncryptedPath,
    );
  }

  textEditorLocator(): Locator {
    return this.page.getByRole('code').locator('.view-line');
  }

  textEditorTextareaLocator(): Locator {
    return this.page.getByRole('code').getByRole('textbox');
  }

  async waitForEditorContent(expectedContent: string) {
    await expect(this.textEditorLocator()).toHaveText(expectedContent);
  }

  async expectContentReloaded() {
    await this.waitForEditorContent(this.currentContent);
  }

  async expectEmptyFolder() {
    await expect(
      this.page.getByRole('heading', { name: 'This folder is empty.' }),
    ).toBeVisible();
  }

  editButtonLocator(): Locator {
    return navbarLocator(this.page).getByRole('link', { name: 'Edit' });
  }

  saveButtonLocator(): Locator {
    return navbarLocator(this.page).getByRole('button', { name: 'Save' });
  }

  deleteButtonLocator(): Locator {
    return navbarLocator(this.page).getByRole('button', { name: 'Delete' });
  }

  async expectViewerOpen() {
    await expect(this.editButtonLocator()).toBeVisible();
  }

  async clickEditButton() {
    await this.editButtonLocator().click();
  }

  async expectNoConflicts() {
    await expect(
      navbarLocator(this.page).getByText(
        'Changes are saved automatically. Last saved',
      ),
    ).toBeVisible();
  }

  async expectDirty() {
    await expect(
      navbarLocator(this.page).getByLabel('File modified'),
    ).toBeVisible();
  }

  async expectNotDirty() {
    await expect(
      navbarLocator(this.page).getByLabel('File unchanged'),
    ).toBeVisible();
  }

  async openEditorContent(
    encryptedPath: string,
    filename: string,
    isEditing: boolean,
    content: string,
  ) {
    await this.editorNavigate(encryptedPath, isEditing);
    await this.expectHeaderName(filename);
    await this.waitForEditorContent(content);
  }

  async viewFile() {
    await this.createFile();
    await this.openEditorContent(
      this.currentEncryptedPath,
      this.currentName,
      false,
      this.currentContent,
    );
  }

  async viewFileFromParent() {
    await this.createFile();
    await this.openParentFolder();
    await filesTableClickFile(this.page, 'File', this.currentName);
    await this.expectHeaderName(this.currentName);
    await this.waitForEditorContent(this.currentContent);
  }

  async viewNonexistentFile() {
    await this.editorNavigate(`/${this.encryptName('nonexistent')}`, false);
    await this.expectHeaderName('nonexistent');
  }

  async editFile() {
    await this.createFile();
    await this.openEditorContent(
      this.currentEncryptedPath,
      this.currentName,
      true,
      this.currentContent,
    );
  }

  async editFileFromFolder() {
    await filesTableSelectFile(this.page, 'File', this.currentName);
    await toolbarEditTextFile(this.page);
  }

  async editFileFromParent() {
    await this.createFile();
    await this.openParentFolder();
    await this.editFileFromFolder();
  }

  async editorAppendText(text: string) {
    await this.textEditorLocator().click();
    // we need type (deprecated) here, fill does not work
    await this.textEditorLocator().type(text);
    this.currentContent += text;
  }

  async changeContent() {
    await this.editorAppendText('1');
  }

  async clickX() {
    await navbarCloseLocator(this.page).click();
  }

  async clickLogo() {
    await navbarLogoLocator(this.page).click();
  }

  async clickSave() {
    await this.saveButtonLocator().isEnabled({ timeout: 1000 });
    await this.saveButtonLocator().click();
  }

  async clickDelete() {
    await this.deleteButtonLocator().isEnabled({ timeout: 1000 });
    await this.deleteButtonLocator().click();
  }

  async goBack() {
    await this.page.goBack();
  }

  async closeTab() {
    await this.page.close();
  }

  async expectFilesRootOpen() {
    await expect(breadcrumbsLocator(this.page)).toHaveText('My safe box');
  }

  async expectFileBrowserFile() {
    await expect(
      filesTableRowNameLocator(this.page, 'File', this.currentName),
    ).toBeVisible();
  }

  async expectFileBrowserFileSelected() {
    await filesTableFileSelected(this.page, 'File', this.currentName);
  }

  async expectServerContent(expectedContent: string) {
    await this.webVaultClient.waitForFileContent(
      this.repo,
      this.currentEncryptedPath,
      expectedContent,
      15000,
    );
  }

  async expectServerContentMatch() {
    await this.expectServerContent(this.currentContent);
  }

  async expectServerContentOld() {
    await this.expectServerContent(this.serverContent);
  }

  async changeContentOnServer() {
    this.currentContent = 'editorcontent2';
    this.serverContent = this.currentContent;

    await this.webVaultClient.setFileContent(
      this.repo,
      this.currentEncryptedPath,
      'txt',
      this.currentContent,
      15000,
    );
  }

  async renameFileOnServer() {
    const name = 'file renamed.txt';
    await this.webVaultClient.renameFile(
      this.repo,
      this.currentEncryptedPath,
      name,
      15000,
    );
    this.setCurrentName(name);
  }

  async deleteFileOnServer() {
    await this.webVaultClient.deleteFile(
      this.repo,
      this.currentEncryptedPath,
      15000,
    );
  }

  async hasOnBeforeLeave() {
    return await this.page.evaluate(() => window.onbeforeunload != null);
  }

  async expectHasOnBeforeLeave() {
    expect(await this.hasOnBeforeLeave()).toBeTruthy();
  }

  async clearOnBeforeLeave() {
    await this.page.evaluate(() => {
      window.onbeforeunload = null;
    });
  }
}

export const textEditorTest = test.extend<{ textEditor: TextEditor }>({
  textEditor: async ({ page, debugClient, webVaultClient, dialogs }, use) => {
    await createRepo(debugClient);

    await webVaultClient.load();

    const repo = await webVaultClient.waitForRepo();

    await webVaultClient.unlockRepo(repo);

    const textEditor = new TextEditor(page, webVaultClient, repo, dialogs);

    await use(textEditor);
  },
});
