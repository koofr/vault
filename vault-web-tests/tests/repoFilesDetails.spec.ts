import { expect } from '@playwright/test';

import { openApp } from '../helpers/app';
import { test } from '../helpers/base';
import { filesTableClickFile, uploadFile } from '../helpers/repoFiles';
import { createRepo, unlockRepo } from '../helpers/repos';
import {
  fileDetailsNameLocator,
  pdfViewerBodyLocator,
  pdfViewerLocator,
  textEditorTest,
} from '../helpers/reposFilesDetails';
import { sleep } from '../helpers/time';

test.describe('repoFilesDetails', () => {
  test.describe('text editor', () => {
    const test = textEditorTest;

    test.describe('View', () => {
      test('View, close (X)', async ({ textEditor }) => {
        await textEditor.viewFile();
        await textEditor.clickX();
        await textEditor.expectFilesRootOpen();
      });

      test('View, close (logo)', async ({ textEditor }) => {
        await textEditor.viewFile();
        await textEditor.clickLogo();
        await textEditor.expectFilesRootOpen();
      });

      test('View, go back', async ({ textEditor }) => {
        await textEditor.openApp();
        await textEditor.viewFile();
        await textEditor.goBack();
        await textEditor.expectFilesRootOpen();
      });

      test('View, close tab', async ({ textEditor }) => {
        await textEditor.viewFile();
        await textEditor.closeTab();
      });

      test('View non-existent', async ({ textEditor }) => {
        await textEditor.viewNonexistentFile();
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
        await textEditor.expectEmptyNavbarNav();
      });
    });

    test.describe('Edit', () => {
      test('Edit, close (X)', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.clickX();
        await textEditor.expectViewerOpen();
      });

      test('Edit, close (logo)', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.clickLogo();
        await textEditor.expectFilesRootOpen();
      });

      test('Open folder, edit, go back', async ({ textEditor }) => {
        await textEditor.editFileFromParent();
        await textEditor.goBack();
        await textEditor.expectFileBrowserFile();
      });

      test('Edit, close tab', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.closeTab();
      });

      test('Edit non-existent', async ({ textEditor }) => {
        await textEditor.viewNonexistentFile();
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
        await textEditor.expectEmptyNavbarNav();
      });
    });

    test.describe('Edit and change', () => {
      test('Edit, change, close (X)', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.clickX();
        await textEditor.expectServerContentMatch();
        await textEditor.expectViewerOpen();
      });

      test('Edit, change, close (logo)', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.clickLogo();
        await textEditor.expectServerContentMatch();
        await textEditor.expectFilesRootOpen();
      });

      test('Edit, close (logo), upload error, retry, error, retry', async ({
        textEditor,
        debugClient,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await debugClient.withQueue(
          async (request) => {
            if (/files\/put/.test(request.url)) {
              await debugClient.queueNext(500);

              return false;
            } else {
              await debugClient.queueNext();

              return true;
            }
          },
          async () => {
            await textEditor.clickLogo();
          },
        );
        await textEditor.dialogs.waitForDialog(
          'File could not be saved',
          /File could not be saved \(.*\)\. Do you want to Try again or Discard the changes\?/,
          'Try again',
          'Discard changes',
        );
        await debugClient.withQueue(
          async (request) => {
            if (/files\/put/.test(request.url)) {
              await debugClient.queueNext(500);

              return false;
            } else {
              await debugClient.queueNext();

              return true;
            }
          },
          async () => {
            await textEditor.dialogs.clickButtonWait('Try again');
          },
        );
        await textEditor.dialogs.waitForDialog(
          'File could not be saved',
          /File could not be saved \(.*\)\. Do you want to Try again or Discard the changes\?/,
          'Try again',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Try again');
        await textEditor.expectServerContentMatch();
      });

      test('Edit, close (logo), upload error, discard changes', async ({
        textEditor,
        debugClient,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await debugClient.withQueue(
          async (request) => {
            if (/files\/put/.test(request.url)) {
              await debugClient.queueNext(500);

              return false;
            } else {
              await debugClient.queueNext();

              return true;
            }
          },
          async () => {
            await textEditor.clickLogo();
          },
        );
        await textEditor.dialogs.waitForDialog(
          'File could not be saved',
          /File could not be saved \(.*\)\. Do you want to Try again or Discard the changes\?/,
          'Try again',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Discard changes');
        await sleep(200);
        await textEditor.dialogs.waitForHidden();
        textEditor.currentContent = 'editorcontent';
        await textEditor.expectServerContentMatch();
      });

      test('Edit, change, go back', async ({ textEditor }) => {
        await textEditor.editFileFromParent();
        await textEditor.changeContent();
        await textEditor.goBack();
        await textEditor.expectServerContentMatch();
        await textEditor.expectFileBrowserFile();
      });

      test('Edit, change, close tab', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.expectHasOnBeforeLeave();
        await textEditor.clearOnBeforeLeave();
        await textEditor.closeTab();
        await textEditor.expectServerContentOld();
      });

      test('Edit, change, autosave', async ({ textEditor }) => {
        textEditor.autosaveMs = 200;
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.expectServerContentMatch();
      });
    });

    test.describe('Edit, change, save', () => {
      test('Edit, change, save, close (X)', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.clickSave();
        await textEditor.expectServerContentMatch();
        await textEditor.clickX();
        await textEditor.expectViewerOpen();
      });

      test('Edit, change, save, close (logo)', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.clickSave();
        await textEditor.expectServerContentMatch();
        await textEditor.clickLogo();
        await textEditor.expectFilesRootOpen();
      });

      test('Edit, change, save, go back', async ({ textEditor }) => {
        await textEditor.editFileFromParent();
        await textEditor.changeContent();
        await textEditor.clickSave();
        await textEditor.expectServerContentMatch();
        await textEditor.goBack();
        await textEditor.expectFileBrowserFile();
      });

      test('Edit, change, save, close tab', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.clickSave();
        await textEditor.expectServerContentMatch();
        await textEditor.closeTab();
      });
    });

    test.describe('Edit, change, save (server change)', () => {
      test('View, edit, change, save', async ({ textEditor }) => {
        await textEditor.viewFile();
        await textEditor.changeContentOnServer();
        await textEditor.expectContentReloaded();
      });

      test('Edit, edit, change, save', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContentOnServer();
        await textEditor.expectContentReloaded();
      });
    });

    test.describe('Edit, change, edit, change, save (server change)', () => {
      test('Edit, change, edit, change, save', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
      });

      test('Edit, change, edit, change, save, save, cancel', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickSave();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file?',
          'Save as a new file',
          'Cancel',
        );
        await textEditor.dialogs.clickButtonWait('Cancel');
        await textEditor.expectHeaderErrorStays(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
      });

      test('Edit, change, edit, change, save, save, save as new file', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickSave();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file?',
          'Save as a new file',
          'Cancel',
        );
        await textEditor.dialogs.clickButtonWait('Save as a new file');
        textEditor.autorenamed();
        await textEditor.expectServerContentMatch();
        await textEditor.expectHeaderNameMatch();
        await textEditor.expectURLPathMatch();
        await textEditor.expectNoConflicts();
        await textEditor.expectNotDirty();
      });
    });

    test.describe('Edit, change, edit, change, close (server change)', () => {
      test('Edit, change, edit, change, close (X), discard changes', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickX();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file or Discard them?',
          'Save as a new file',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Discard changes');
        await textEditor.expectViewerOpen();
        await textEditor.expectContentReloaded();
      });

      test('Edit, change, edit, change, close (X), save as new file', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickX();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file or Discard them?',
          'Save as a new file',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Save as a new file');
        textEditor.autorenamed();
        await textEditor.expectViewerOpen();
        await textEditor.expectServerContentMatch();
        await textEditor.expectHeaderNameMatch();
        await textEditor.expectURLPathMatch();
      });

      test('Edit, change, edit, change, close (logo)', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickLogo();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file or Discard them?',
          'Save as a new file',
          'Discard changes',
        );
      });

      test('Edit, change, edit, change, close (logo), discard changes', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickLogo();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file or Discard them?',
          'Save as a new file',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Discard changes');
        await textEditor.expectFilesRootOpen();
      });

      test('Edit, change, edit, change, close (logo), save as new file', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.clickLogo();
        await textEditor.dialogs.waitForDialog(
          'File was changed by someone else since your last save',
          'Saving into the existing file is not possible. Do you want to Save your changes as a new file or Discard them?',
          'Save as a new file',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Save as a new file');
        textEditor.autorenamed();
        await textEditor.expectServerContentMatch();
      });

      test('Edit, change, edit, change, close tab', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.changeContentOnServer();
        await textEditor.expectHeaderError(
          'File was changed by someone else since your last save. Automatic saving is disabled.',
        );
        await textEditor.expectHasOnBeforeLeave();
        await textEditor.clearOnBeforeLeave();
        await textEditor.closeTab();
        await textEditor.expectServerContentMatch();
      });
    });

    test.describe('Rename (server change)', () => {
      test('View, rename', async ({ textEditor }) => {
        await textEditor.viewFile();
        await textEditor.renameFileOnServer();
        await textEditor.expectHeaderNameMatch();
        await textEditor.expectURLPathMatch();
      });

      test('Edit, rename', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.renameFileOnServer();
        await textEditor.expectHeaderNameMatch();
        await textEditor.expectURLPathMatch();
      });

      test('Edit, change, rename', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.renameFileOnServer();
        await textEditor.expectHeaderNameMatch();
        await textEditor.expectURLPathMatch();
        await textEditor.expectDirty();
      });

      test('Edit, change, rename, save', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.renameFileOnServer();
        await textEditor.expectHeaderNameMatch();
        await textEditor.expectURLPathMatch();
        await textEditor.expectDirty();
        await textEditor.clickSave();
        await textEditor.expectNotDirty();
        await textEditor.expectServerContentMatch();
      });
    });

    test.describe('View, delete', () => {
      test('View, delete, cancel', async ({ textEditor }) => {
        await textEditor.viewFile();
        await textEditor.clickDelete();
        await textEditor.dialogs.waitForDialog(
          'Delete files',
          'Do you really want to delete 1 item?',
          'Delete',
          'Cancel',
        );
        await textEditor.dialogs.clickButtonWait('Cancel');
      });

      test('View, delete, delete', async ({ textEditor }) => {
        await textEditor.viewFileFromParent();
        await textEditor.clickDelete();
        await textEditor.dialogs.waitForDialog(
          'Delete files',
          'Do you really want to delete 1 item?',
          'Delete',
          'Cancel',
        );
        await textEditor.dialogs.clickButtonWait('Delete');
        await textEditor.expectEmptyFolder();
        // await textEditor.expectAlertNotToAppear()
      });
    });

    test.describe('Edit, change, delete (server change)', () => {
      test('Edit, change, delete', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.deleteFileOnServer();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it.`,
          'Ok',
        );
        await textEditor.dialogs.clickButtonWait('Ok');
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
      });

      test('Edit, change, delete, save, cancel', async ({ textEditor }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.deleteFileOnServer();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it.`,
          'Ok',
        );
        await textEditor.dialogs.clickButtonWait('Ok');
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
        await textEditor.clickSave();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location?`,
          'Save to a new location',
          'Cancel',
        );
        await textEditor.dialogs.clickButtonWait('Cancel');
        await textEditor.expectHeaderErrorStays(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
      });

      test('Edit, change, delete, save, save to a new location', async ({
        textEditor,
      }) => {
        await textEditor.editFile();
        await textEditor.changeContent();
        await textEditor.deleteFileOnServer();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it.`,
          'Ok',
        );
        await textEditor.dialogs.clickButtonWait('Ok');
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
        await textEditor.clickSave();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location?`,
          'Save to a new location',
          'Cancel',
        );
        await textEditor.dialogs.clickButtonWait('Save to a new location');
        await textEditor.dialogs.waitForDialog(
          'File location changed',
          `File ${textEditor.currentName} was saved here because it could not be saved in its original location.`,
          'Ok',
        );
        await textEditor.dialogs.clickButtonWait('Ok');
        await textEditor.expectFilesRootOpen();
        await textEditor.expectFileBrowserFileSelected();
      });

      test('Edit, change, delete, close (X), discard changes', async ({
        textEditor,
      }) => {
        await textEditor.editFileFromParent();
        await textEditor.changeContent();
        await textEditor.deleteFileOnServer();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it.`,
          'Ok',
        );
        await textEditor.dialogs.clickButtonWait('Ok');
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
        await textEditor.clickX();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it. Do you want to Save the file to a new location or Discard the changes?`,
          'Save to a new location',
          'Discard changes',
        );
        await textEditor.dialogs.clickButtonWait('Discard changes');
        // it opens parent folder not viewer, because it would just show the same error again
        await textEditor.expectEmptyFolder();
      });

      test('Edit, change, delete, close tab, discard changes', async ({
        textEditor,
      }) => {
        await textEditor.editFileFromParent();
        await textEditor.changeContent();
        await textEditor.deleteFileOnServer();
        await textEditor.dialogs.waitForDialog(
          'File not accessible',
          `File ${textEditor.currentName} is no longer accessible. Probably it was deleted or you no longer have access to it.`,
          'Ok',
        );
        await textEditor.dialogs.clickButtonWait('Ok');
        await textEditor.expectHeaderError(
          'This file is no longer accessible. Probably it was deleted or you no longer have access to it.',
        );
        await textEditor.expectHasOnBeforeLeave();
        await textEditor.clearOnBeforeLeave();
        await textEditor.closeTab();
      });
    });
  });

  test.describe('PDF viewer', () => {
    test('open PDF viewer', async ({ page, debugClient }) => {
      await createRepo(debugClient);
      await openApp(page);
      await unlockRepo(page);

      await uploadFile(page, 'test-files/example.pdf');

      await filesTableClickFile(page, 'File', 'example.pdf');

      await expect(fileDetailsNameLocator(page)).toHaveText('example.pdf');

      await expect(pdfViewerLocator(page)).toBeVisible();

      await expect(await pdfViewerBodyLocator(page)).toHaveText(
        /Example PDF file/,
        { timeout: 30000 },
      );
    });
  });
});
