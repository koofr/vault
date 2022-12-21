/* eslint-disable */
// https://www.w3.org/TR/2012/WD-file-system-api-20120417/#idl-def-Entry
interface Metadata {
  modificationTime: Date;
  size: number;
}

interface Flags {
  create?: boolean;
  exclusive?: boolean;
}

interface DirectoryReader {
  readEntries(
    successCallback: (entries: Entry[]) => void,
    errorCallback?: (err: Error) => void
  ): void;
}

interface Entry {
  readonly isFile: boolean;
  readonly isDirectory: boolean;
  readonly name: string;
  readonly fullPath: string;
  getMetadata(
    successCallback: (metadata: Metadata) => void,
    errorCallback?: (err: Error) => void
  );
  moveTo(
    parent: DirectoryEntry,
    newName?: string,
    successCallback?: (entry: Entry) => void,
    errorCallback?: (err: Error) => void
  ): void;
  copyTo(
    parent: DirectoryEntry,
    newName?: string,
    successCallback?: (entry: Entry) => void,
    errorCallback?: (err: Error) => void
  ): void;
  toURL(): string;
  remove(successCallback: () => void, errorCallback?: (err: Error) => void);
  getParent(
    successCallback: (entry: Entry) => void,
    errorCallback?: (err: Error) => void
  );
}

interface DirectoryEntry extends Entry {
  readonly isFile: false;
  readonly isDirectory: true;
  createReader(): DirectoryReader;
  getFile(
    path: string,
    options?: Flags,
    successCallback?: (entry: Entry) => void,
    errorCallback?: (err: Error) => void
  );
  getDirectory(
    path: string,
    options?: Flags,
    successCallback?: (entry: Entry) => void,
    errorCallback?: (err: Error) => void
  );
  removeRecursively(
    successCallback: () => void,
    errorCallback?: (err: Error) => void
  );
}

interface FileEntry extends Entry {
  isFile: true;
  isDirectory: false;
  file(
    successCallback: (file: File) => void,
    errorCallback?: (err: Error) => void
  ): void;
}

type WebkitGetAsEntry = DirectoryEntry | FileEntry;
