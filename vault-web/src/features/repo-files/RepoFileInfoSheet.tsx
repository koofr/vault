import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import { ModalClose, ModalCloseContext } from '../../components/modal/Modal';
import { useNavbarSticky } from '../../components/navbar/NavbarSticky';
import { RepoFile } from '../../vault-wasm/vault-wasm';
import { useSubscribe } from '../../webVault/useSubscribe';

import { TRANSFERS_SUMMARY_HEIGHT } from '../transfers/TransfersSummary';

import { RepoFileInfoGeneral } from './RepoFileInfoGeneral';
import { RepoFileInfoImage } from './RepoFileInfoImage';

export const RepoFileInfoSheet = memo<{
  file?: RepoFile;
  hide: () => void;
}>(({ file, hide }) => {
  const theme = useTheme();
  const isVisible = file !== undefined;
  const isSticky = useNavbarSticky();
  const [transfersIsActive] = useSubscribe(
    (v, cb) => v.transfersIsActiveSubscribe(cb),
    (v) => v.transfersIsActiveData,
    [],
  );
  const bottom = transfersIsActive ? TRANSFERS_SUMMARY_HEIGHT : 0;

  return (
    <div
      className={cx(
        css`
          flex-direction: column;
          position: fixed;
          background-color: #fff;
          z-index: ${theme.zindex.fileInfoSheet};
          display: flex;
          top: 70px;
          bottom: ${bottom}px;
          left: -250px;
          width: 250px;
          border-right: 1px solid ${theme.colors.borderLight};
          border-top: 1px solid ${theme.colors.borderLight};
          transition: left 0.3s ease-out;
        `,
        isVisible &&
          css`
            left: 0;
          `,
        isSticky &&
          css`
            top: 69px;
          `,
      )}
    >
      {file !== undefined ? (
        <div
          className={css`
            display: flex;
            flex-direction: column;
            flex-grow: 1;
            overflow: hidden;
          `}
        >
          <div
            className={css`
              display: flex;
              flex-direction: row;
              align-items: center;
              flex-shrink: 0;
              border-bottom: 1px solid ${theme.colors.border};
              height: 46px;
            `}
          >
            <div
              className={css`
                flex-grow: 1;
                font-size: 14px;
                font-weight: normal;
                margin: 0 0 0 20px;
                color: ${theme.colors.text};
              `}
            >
              Info
            </div>

            <div
              className={css`
                margin-right: 7px;
              `}
            >
              <ModalCloseContext.Provider value={hide}>
                <ModalClose />
              </ModalCloseContext.Provider>
            </div>
          </div>
          <div
            className={css`
              display: flex;
              flex-direction: column;
              flex-grow: 1;
              padding: 20px 25px 0;
              overflow-x: hidden;
            `}
          >
            <RepoFileInfoGeneral file={file} />
          </div>
          <RepoFileInfoImage file={file} />
        </div>
      ) : null}
    </div>
  );
});
