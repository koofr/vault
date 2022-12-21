import format from 'date-fns/format';
import throttle from 'lodash/throttle';

import {
  entryIsDirectory,
  directoryEntries,
  entryFile,
  webkitFilesToEntries,
} from '../../utils/domFiles';
import { normalizeFilename } from '../../utils/normalizeFilename';

export function normalizePath(path: string): string {
  if (path === '' || path === '/') {
    return '/';
  }
  if (path[path.length - 1] === '/') {
    return path.slice(0, path.length - 1);
  }
  return path;
}

export function joinPaths(parentPath: string, path: string): string {
  parentPath = normalizePath(parentPath);
  path = normalizePath(path);

  if (path === '/') {
    return parentPath;
  }

  return parentPath + (/\/$/.test(parentPath) ? '' : '/') + path.slice(1);
}

export function joinPath(path: string, name: string): string {
  return (
    path +
    (/\/$/.test(path) ? '' : '/') +
    (name[0] !== '/' ? name : name.slice(1))
  );
}

export type RelPath = string;

export interface UploadEntry {
  parentPath: RelPath;
  name: string;
  file: File;
}

interface UploadFilesQueueEntry {
  entry: UploadEntry;
  resolve: () => void;
  reject: (reason: any) => void;
}

export interface UploadsHelperOptions {
  upload: (entries: UploadEntry[]) => Promise<void>[];
}

export class UploadsHelper {
  private upload: (entries: UploadEntry[]) => Promise<void>[];
  private queue: UploadFilesQueueEntry[] = [];

  constructor(options: UploadsHelperOptions) {
    this.upload = options.upload;
  }

  uploadFileEntries(entries: UploadEntry[]): Promise<void>[] {
    if (entries.length === 0) {
      return [];
    }

    return this.upload(entries);
  }

  private getFileName(file: File): string {
    let name = normalizeFilename(file.name);

    // TODO this should only be done for clipboard, but we need to do some
    // refactoring first

    // Old versions of iOS Safari named all photos image.jpeg. Browsers also use
    // image.jpg and image.png for image clipboard paste.
    if (name === 'image.jpeg' || name === 'image.jpg' || name === 'image.png') {
      const dateNow = format(new Date(), 'yyyyMMdd_hhmmssSSS');
      const ext = name.split('.').pop();
      name = `image_${dateNow}.${ext}`;
    }

    return name;
  }

  private addEntry(entry: UploadEntry): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      // TODO make a list of ignored files
      // .DS_Store
      // .DS_Store?
      // ._*
      // .Spotlight-V100
      // .Trashes
      // ehthumbs.db
      // Thumbs.db
      if (entry.name === '.DS_Store') {
        resolve();
        return;
      }

      this.queue.push({
        entry: entry,
        resolve: resolve,
        reject: reject,
      });

      this.processQueueThrottled();
    });
  }

  private processQueue = () => {
    if (this.queue.length === 0) {
      return;
    }

    // create a copy of queue
    const entries = this.queue.slice();
    // clear the queue
    this.queue.length = 0;

    const fileEntries = entries.map((entry) => entry.entry);

    this.uploadFileEntries(fileEntries).forEach((p, i) => {
      p.then(entries[i].resolve, entries[i].reject);
    });
  };

  // throttle and batch file uploads while iterating directories to prevent
  // browser freezing
  private processQueueThrottled = throttle(this.processQueue, 100);

  private async handleEntryDir(
    entry: DirectoryEntry,
    parentPath: RelPath
  ): Promise<RelPath> {
    const name = normalizeFilename(entry.name);
    const path = joinPath(parentPath, name);

    // wait a bit while iterating directories to prevent browser from freezing
    await wait(30);

    const entries = await directoryEntries(entry);

    await Promise.all(
      entries.map((entry) =>
        this.handleEntry(entry as DirectoryEntry | FileEntry, path).then(
          () => {}
        )
      )
    );

    return path;
  }

  private async handleEntryFile(
    fileEntry: FileEntry,
    parentPath: RelPath
  ): Promise<void> {
    const file = await entryFile(fileEntry);

    const uploadEntry: UploadEntry = {
      parentPath: parentPath,
      name: this.getFileName(file),
      file: file,
    };

    await this.addEntry(uploadEntry);
  }

  private async handleEntry(
    entry: DirectoryEntry | FileEntry,
    parentPath: RelPath
  ): Promise<void> {
    if (entryIsDirectory(entry)) {
      await this.handleEntryDir(entry, parentPath);
    } else {
      await this.handleEntryFile(entry, parentPath);
    }
  }

  // Upload using drag'n'drop file or directory (Chrome Desktop, Firefox, Safari macOS)
  private uploadDataTransferItems(items: DataTransferItem[]): Promise<void>[] {
    return items
      .filter((item) => item.webkitGetAsEntry != null) // could be null or undefined
      .map((item) => (item as any).webkitGetAsEntry() as WebkitGetAsEntry)
      .filter((entry) => entry != null) // could be null or undefined
      .map((entry) => this.handleEntry(entry, '/'));
  }

  // Upload using file input files or folders (Chrome Desktop, Firefox, Safari macOS, Safari iOS, Chrome iOS, Edge)
  private uploadWebkitFilesAsEntries(files: File[]): Promise<void>[] {
    return webkitFilesToEntries(files).map((entry) =>
      this.handleEntry(entry as DirectoryEntry | FileEntry, '/')
    );
  }

  // Upload using file input files (IE11, old Androids, old Safari iOS)
  private uploadNativeFiles(files: File[]): Promise<void>[] {
    return this.uploadFileEntries(
      files.map((file) => ({
        parentPath: '/',
        name: this.getFileName(file),
        file: file,
      }))
    );
  }

  uploadFiles(files: File[] | DataTransferItem[]): Promise<void>[] {
    if (files.length > 0) {
      const testFile = files[0] as any;
      if (testFile.webkitGetAsEntry != null) {
        // could be null or undefined
        if (testFile.webkitGetAsEntry() != null) {
          return this.uploadDataTransferItems(files as DataTransferItem[]);
        }
        return [];
      }
      if (testFile.webkitRelativePath != null) {
        // could be null or undefined
        return this.uploadWebkitFilesAsEntries(files as File[]);
      }
    }

    return this.uploadNativeFiles(files as File[]);
  }
}

const wait = (duration: number): Promise<void> =>
  new Promise((resolve) => setTimeout(resolve, duration));
