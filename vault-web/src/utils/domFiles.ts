export function canUploadFolder() {
  const input = document.createElement('input');

  return (input as any).webkitdirectory !== undefined;
}

export function entryIsDirectory(entry: Entry): entry is DirectoryEntry {
  return entry.isDirectory;
}

export function directoryEntries(entry: DirectoryEntry): Promise<Entry[]> {
  return new Promise<Entry[]>((resolve, reject) => {
    const dirReader = entry.createReader();

    let entries: Entry[] = [];

    const readEntries = () => {
      dirReader.readEntries((results) => {
        if (results.length === 0) {
          resolve(entries);
        } else {
          entries = entries.concat(Array.from(results));

          setTimeout(readEntries, 10);
        }
      }, reject);
    };

    readEntries();
  });
}

export function entryFile(entry: FileEntry): Promise<File> {
  return new Promise<File>((resolve, reject) => entry.file(resolve, reject));
}

export function webkitFilesToEntries(files: File[]): Entry[] {
  const createFileEntry = (file: File): FileEntry => {
    const fileFn = (
      successCallback: (file: File) => void,
      errorCallback?: (err: Error) => void,
    ): void => {
      successCallback(file);
    };

    return {
      isDirectory: false,
      isFile: true,
      name: file.name,
      file: fileFn,
    } as FileEntry;
  };

  const createDirectoryEntry = (
    name: String,
    children: Entry[],
  ): DirectoryEntry => {
    const createReader = (): DirectoryReader => {
      let readDone = false;

      const readEntries = (
        successCallback: (entries: Entry[]) => void,
        errorCallback?: (err: Error) => void,
      ): void => {
        if (readDone) {
          successCallback([]);
        } else {
          readDone = true;

          successCallback(children);
        }
      };

      return {
        readEntries: readEntries,
      };
    };

    return {
      name: name,
      isFile: false,
      isDirectory: true,
      createReader: createReader,
    } as DirectoryEntry;
  };

  interface Part {
    name: string;
    isDirectory: boolean;
    file: File | undefined;
    children: Map<string, Part> | undefined;
  }

  const rootPart: Part = {
    name: '',
    isDirectory: true,
    file: undefined,
    children: new Map<string, Part>(),
  };

  files.forEach((file) => {
    const pathParts: string[] = (file as any).webkitRelativePath.split('/');
    const parentPathParts = pathParts.slice(0, pathParts.length - 1);

    let parentPart = rootPart;

    parentPathParts.forEach((partName) => {
      let part = parentPart.children!.get(partName);

      if (part === undefined) {
        part = {
          name: partName,
          isDirectory: true,
          file: undefined,
          children: new Map<string, Part>(),
        };

        parentPart.children!.set(partName, part);
      }

      parentPart = part;
    });

    parentPart.children!.set(file.name, {
      name: file.name,
      isDirectory: false,
      file: file,
      children: undefined,
    });
  });

  const partToEntry = (part: Part): Entry => {
    if (part.isDirectory) {
      const children = Array.from(part.children!.values()).map(partToEntry);

      return createDirectoryEntry(part.name, children);
    }

    return createFileEntry(part.file!);
  };

  return Array.from(rootPart.children!.values()).map(partToEntry);
}
