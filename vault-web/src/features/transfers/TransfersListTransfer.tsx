import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';

import { Button } from '../../components/Button';
import { FileIcon } from '../../components/file-icon/FileIcon';
import { Transfer } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

export const TransfersListTransfer = memo<{ transfer: Transfer }>(
  ({ transfer }) => {
    const { id, name, fileIconAttrs, state } = transfer;
    const theme = useTheme();
    const webVault = useWebVault();
    const retry = useCallback(() => {
      webVault.transfersRetry(id);
    }, [webVault, id]);
    const abort = useCallback(() => {
      webVault.transfersAbort(id);
    }, [webVault, id]);

    let text = '';

    switch (state.type) {
      case 'Waiting':
        text = 'is waiting to be transferred.';
        break;
      case 'Processing':
        text = 'is being processed.';
        break;
      case 'Transferring':
        text = 'is being transferred.';
        break;
      case 'Failed':
        text = `failed. ${state.error}`;
        break;
      case 'Done':
        text = 'has been transferred.';
        break;
    }

    return (
      <div
        className={css`
          border-bottom: 1px solid ${theme.colors.borderLight};
          padding: ${theme.isMobile ? '0 7px 0 15px' : '0'};
          margin: ${theme.isMobile ? '0' : '0 25px'};
        `}
      >
        <div
          className={css`
            max-width: 840px;
            display: flex;
            align-items: center;
            padding: 9px 0 8px;
            margin: auto;
          `}
        >
          <div
            className={css`
              width: 26px;
              height: 29px;
              margin-right: 15px;
              flex-shrink: 0;
            `}
          >
            <FileIcon size="Sm" attrs={fileIconAttrs} />
          </div>
          <div
            className={css`
              font-size: 13px;
              font-weight: normal;
              color: ${theme.colors.text};
              text-overflow: ellipsis;
              white-space: nowrap;
              overflow: hidden;
            `}
          >
            {name}
          </div>
          <div
            className={css`
              font-size: 13px;
              font-weight: normal;
              color: ${theme.colors.text};
              flex-grow: 1;
              flex-shrink: 0;
              margin-left: 5px;
            `}
          >
            {text}
          </div>
          {transfer.canRetry ? (
            <Button
              type="button"
              variant="primary-inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={retry}
            >
              Retry
            </Button>
          ) : null}
          {transfer.canAbort ? (
            <Button
              type="button"
              variant="destructive-inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={abort}
            >
              Cancel
            </Button>
          ) : null}
        </div>
      </div>
    );
  }
);
