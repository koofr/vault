import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { JSX, memo, ReactNode, useCallback } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

import TransfersClearHoverIcon from '../../assets/images/transfers-clear-hover.svg?react';
import TransfersClearIcon from '../../assets/images/transfers-clear.svg?react';
import { Button } from '../../components/Button';
import { FileIcon } from '../../components/file-icon/FileIcon';
import { buttonReset } from '../../styles/mixins/buttons';
import { Transfer } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

export const TransfersListTransfer = memo<{ transfer: Transfer }>(
  ({ transfer }) => {
    const intl = useIntl();
    const { id, name, fileIconAttrs, state } = transfer;
    const theme = useTheme();
    const webVault = useWebVault();
    const retry = useCallback(() => {
      webVault.transfersRetry(id);
    }, [webVault, id]);
    const abort = useCallback(() => {
      webVault.transfersAbort(id);
    }, [webVault, id]);
    const open = useCallback(() => {
      webVault.transfersOpen(id);
    }, [webVault, id]);

    const nameEl = (
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
    );

    const statusElFn = (chunks: ReactNode[]) => (
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
        {chunks}
      </div>
    );

    let messageEl: JSX.Element;

    switch (state.type) {
      case 'Waiting':
        messageEl = (
          <FormattedMessage
            id="web.transfers.transfer.message.waiting"
            description="Transfer row status when a file is queued and waiting to start."
            defaultMessage="{name}<status> is waiting to be transferred</status>"
            values={{
              name: nameEl,
              status: statusElFn,
            }}
          />
        );
        break;
      case 'Processing':
        messageEl = (
          <FormattedMessage
            id="web.transfers.transfer.message.processing"
            description="Transfer row status when a file is being prepared before upload/download."
            defaultMessage="{name}<status> is being processed</status>"
            values={{
              name: nameEl,
              status: statusElFn,
            }}
          />
        );
        break;
      case 'Transferring':
        messageEl = (
          <FormattedMessage
            id="web.transfers.transfer.message.transferring"
            description="Transfer row status when a file is actively transferring."
            defaultMessage="{name}<status> is being transferred</status>"
            values={{
              name: nameEl,
              status: statusElFn,
            }}
          />
        );
        break;
      case 'Failed':
        messageEl = (
          <FormattedMessage
            id="web.transfers.transfer.message.failed"
            description="Transfer row status when a file transfer failed, including error details."
            defaultMessage="{name}<status> failed. {error}</status>"
            values={{
              name: nameEl,
              status: statusElFn,
              error: state.error,
            }}
          />
        );
        break;
      case 'Done':
        messageEl = (
          <FormattedMessage
            id="web.transfers.transfer.message.done"
            description="Transfer row status when a file has completed transferring."
            defaultMessage="{name}<status> has been transferred</status>"
            values={{
              name: nameEl,
              status: statusElFn,
            }}
          />
        );
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
          {messageEl}
          {transfer.canOpen ? (
            <Button
              type="button"
              variant="primary-inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={open}
            >
              <FormattedMessage
                id="web.transfers.transfer.open.button"
                description="Button label in a transfer row to open the transferred item."
                defaultMessage="Open"
              />
            </Button>
          ) : null}
          {transfer.canRetry ? (
            <Button
              type="button"
              variant="primary-inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={retry}
            >
              <FormattedMessage
                id="web.transfers.transfer.retry.button"
                description="Button label in a transfer row to retry a failed transfer."
                defaultMessage="Retry"
              />
            </Button>
          ) : null}
          {transfer.state.type === 'Done' ? (
            <button
              type="button"
              className={css`
                ${buttonReset}
                width: 32px;
                height: 32px;
                flex-shrink: 0;
              `}
              onClick={abort}
              aria-label={intl.formatMessage({
                id: 'web.transfers.transfer.clear.aria_label',
                description:
                  'Accessibility label for the icon button that clears a completed transfer row.',
                defaultMessage: 'Clear',
              })}
            >
              <div
                className={css`
                  display: flex;
                  justify-content: center;
                  align-items: center;
                `}
              >
                <TransfersClearIcon
                  className={css`
                    button:hover > div > & {
                      display: none;
                    }
                  `}
                  role="img"
                />
                <TransfersClearHoverIcon
                  className={css`
                    display: none;

                    button:hover > div > & {
                      display: inline;
                    }
                  `}
                  role="img"
                />
              </div>
            </button>
          ) : (
            <Button
              type="button"
              variant="destructive-inline"
              className={css`
                flex-shrink: 0;
              `}
              onClick={abort}
            >
              <FormattedMessage
                id="web.transfers.transfer.cancel.button"
                description="Button label in a transfer row to cancel an in-progress transfer."
                defaultMessage="Cancel"
              />
            </Button>
          )}
        </div>
      </div>
    );
  },
);
TransfersListTransfer.displayName = 'TransfersListTransfer';
