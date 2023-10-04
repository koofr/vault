import { memo, useEffect } from 'react';
import { useDrop } from 'react-dnd';
import { NativeTypes } from 'react-dnd-html5-backend';

import { DropZone } from '../../components/dnd/DropZone';
import { useClipboardUpload } from '../../utils/useClipboardUpload';

import { useUploadFiles } from '../transfers/useUploadFiles';

export const RepoFilesDropZoneComponent = memo(() => {
  const canUpload = true;
  const uploadFiles = useUploadFiles();
  const [{ canDrop, isOver }, drop] = useDrop(
    () => ({
      accept: [NativeTypes.FILE],
      collect: (monitor) => ({
        canDrop: monitor.canDrop(),
        isOver: monitor.isOver({ shallow: true }),
      }),
      drop: (item: any, monitor) => {
        if (monitor.didDrop() || !canUpload) {
          return;
        }

        // item.items can be null or undefined
        const files =
          item.items != null
            ? (Array.from(item.items) as DataTransferItem[])
            : (Array.from(item.files) as File[]);

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
