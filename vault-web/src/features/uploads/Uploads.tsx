import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback, useState } from 'react';

import { useSubscribe } from '../../webVault/useSubscribe';

import { UploadsFiles } from './UploadsFiles';
import { UploadsSummary } from './UploadsSummary';

export const Uploads = memo(() => {
  const theme = useTheme();
  const [areDetailsVisible, setDetailsVisible] = useState(false);
  const toggleDetailsVisible = useCallback(
    () => setDetailsVisible((visible) => !visible),
    []
  );
  const isActive = useSubscribe(
    (v, cb) => v.uploadsIsActiveSubscribe(cb),
    (v) => v.uploadsIsActiveData,
    []
  );

  if (!isActive) {
    return null;
  }

  return (
    <div
      className={css`
        display: block;
        position: fixed;
        left: 0;
        right: 0;
        bottom: 0;
        z-index: ${theme.zindex.uploads};
        border-top: 1px solid ${theme.colors.border};
        background-color: #fff;
      `}
    >
      <UploadsSummary
        areDetailsVisible={areDetailsVisible}
        toggleDetailsVisible={toggleDetailsVisible}
      />
      <div
        className={cx(
          css`
            height: 0px;
            transition: height 0.3s ease-out;
            padding: 2px 0 0;
          `,
          areDetailsVisible &&
            css`
              height: 191px;
              overflow-y: auto;
            `
        )}
      >
        {areDetailsVisible ? <UploadsFiles /> : null}
      </div>
    </div>
  );
});
