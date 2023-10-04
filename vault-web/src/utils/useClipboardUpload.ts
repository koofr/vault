import { useCallback, useEffect } from 'react';

import { isFocusedElementInput } from './isFocusedElementInput';

export function useClipboardUpload(
  uploadFiles: (files: File[] | DataTransferItem[]) => Promise<void>[],
) {
  const onPaste = useCallback(
    (event: ClipboardEvent) => {
      if (isFocusedElementInput()) {
        return;
      }
      // could be null or undefined
      if (event.clipboardData == null) {
        return;
      }
      if (
        event.clipboardData.items == null &&
        event.clipboardData.files == null
      ) {
        return;
      }
      const files =
        event.clipboardData.files != null
          ? Array.from(event.clipboardData.files)
          : // we could also support text upload
            Array.from(event.clipboardData.items).filter(
              (item) => item.kind === 'file',
            );
      Promise.all(uploadFiles(files));
    },
    [uploadFiles],
  );
  useEffect(() => {
    window.addEventListener('paste', onPaste as EventListener);
    return () => window.removeEventListener('paste', onPaste as EventListener);
  }, [onPaste]);
}
