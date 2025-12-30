import { memo, useEffect } from 'react';
import { useDrop } from 'react-dnd';
import { NativeTypes } from 'react-dnd-html5-backend';

import { DropZone } from '../../components/dnd/DropZone';
import { useClipboardUpload } from '../../utils/useClipboardUpload';

import { useUploadFiles } from '../transfers/useUploadFiles';

export const RepoFilesDropZoneWeb = memo(() => {
  const canUpload = true;
  const uploadFiles = useUploadFiles();
  const [{ canDrop, isOver }, drop] = useDrop(
    () => ({
      accept: [NativeTypes.FILE],
      collect: (monitor) => ({
        canDrop: monitor.canDrop(),
        isOver: monitor.isOver({ shallow: true }),
      }),
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      drop: (item: any, monitor) => {
        if (monitor.didDrop() || !canUpload) {
          return;
        }

        // item.items can be null or undefined
        const files =
          // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access
          item.items != null
            ? // eslint-disable-next-line @typescript-eslint/no-unsafe-argument, @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unnecessary-type-assertion
              (Array.from(item.items) as DataTransferItem[])
            : // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-unnecessary-type-assertion, @typescript-eslint/no-unsafe-argument
              (Array.from(item.files) as File[]);

        // eslint-disable-next-line @typescript-eslint/no-floating-promises
        Promise.all(uploadFiles(files));
      },
    }),
    [uploadFiles],
  );
  useEffect(() => {
    drop(document.body);
  }, [drop]);

  useClipboardUpload(uploadFiles);

  return <DropZone isActive={canDrop} isOver={isOver} isAllowed={canUpload} />;
});
RepoFilesDropZoneWeb.displayName = 'RepoFilesDropZoneWeb';
