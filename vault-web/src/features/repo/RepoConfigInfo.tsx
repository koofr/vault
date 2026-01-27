import { css } from '@emotion/css';
import { memo, useCallback, useRef, useState } from 'react';
import { FormattedMessage } from 'react-intl';

import { Button } from '../../components/Button';
import { RepoConfig } from '../../vault-wasm/vault-wasm';

export const RepoConfigInfo = memo<{ config: RepoConfig }>(({ config }) => {
  const contentRef = useRef<HTMLDivElement>(null);
  const [copied, setCopied] = useState(false);
  const copy = useCallback(() => {
    if (contentRef.current !== null) {
      const range = document.createRange();

      range.selectNode(contentRef.current);

      const selection = window.getSelection();

      if (selection !== null) {
        selection.removeAllRanges();
        selection.addRange(range);
        document.execCommand('copy');
        selection.removeAllRanges();

        setCopied(true);
      }
    }
  }, []);

  return (
    <div>
      <div
        className={css`
          margin-bottom: 20px;

          & p {
            margin: 0 0 15px;
          }

          & strong {
            font-weight: 600;
          }
        `}
        ref={contentRef}
      >
        <p>
          <FormattedMessage
            id="web.repo_config_info.location"
            description="Label for the storage location line in the Safe Box config summary."
            defaultMessage="<b>Location</b>: {location}"
            values={{
              location: config.location.path,
              b: (chunks) => <strong>{chunks}</strong>,
            }}
          />
        </p>
        <p>
          <FormattedMessage
            id="web.repo_config_info.filename_encryption"
            description="Label for the filename encryption setting in the Safe Box config summary."
            defaultMessage="<b>Filename encryption</b>: {filename_encryption}"
            values={{
              filename_encryption: 'standard',
              b: (chunks) => <strong>{chunks}</strong>,
            }}
          />
        </p>
        <p>
          <FormattedMessage
            id="web.repo_config_info.encrypt_directory_names"
            description="Label for the directory name encryption setting in the Safe Box config summary."
            defaultMessage="<b>Encrypt directory names</b>: {encrypt_directory_names}"
            values={{
              encrypt_directory_names: 'true',
              b: (chunks) => <strong>{chunks}</strong>,
            }}
          />
        </p>
        <p>
          <FormattedMessage
            id="web.repo_config_info.salt"
            description="Label for the salt (password2) line in the Safe Box config summary."
            defaultMessage="<b>Salt (password2)</b>: {salt}"
            values={{
              salt: config.salt,
              b: (chunks) => <strong>{chunks}</strong>,
            }}
          />
        </p>
        <p>
          <FormattedMessage
            id="web.repo_config_info.rclone_config"
            description="Label preceding the raw rclone config block in the Safe Box config summary."
            defaultMessage="<b>rclone config</b>:"
            values={{
              b: (chunks) => <strong>{chunks}</strong>,
            }}
          />
        </p>
        <pre
          className={css`
            word-break: break-all;
            white-space: pre-wrap;
          `}
        >
          <code>{config.rcloneConfig}</code>
        </pre>
        <p
          className={css`
            margin: 0;
          `}
        ></p>
      </div>

      <Button type="button" variant="primary" onClick={copy}>
        {copied ? (
          <FormattedMessage
            id="web.repo_config_info.copy.button.copied"
            description="Button label after the config has been copied to the clipboard."
            defaultMessage="Copied"
          />
        ) : (
          <FormattedMessage
            id="web.repo_config_info.copy.button"
            description="Button label to copy the config summary to the clipboard."
            defaultMessage="Copy"
          />
        )}
      </Button>
    </div>
  );
});
RepoConfigInfo.displayName = 'RepoConfigInfo';
