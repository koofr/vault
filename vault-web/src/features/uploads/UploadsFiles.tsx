import { css } from '@emotion/css';
import { memo } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

import { UploadsFile } from './UploadsFile';

export const UploadsFiles = memo(() => {
  const [uploads] = useSubscribe(
    (v, cb) => v.uploadsFilesSubscribe(cb),
    (v) => v.uploadsFilesData,
    []
  );

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
      `}
    >
      {uploads.files.map((file) => (
        <UploadsFile key={file.id} file={file} />
      ))}
    </div>
  );
});
