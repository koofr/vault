import { css } from '@emotion/css';
import { memo, useCallback, useRef, useState } from 'react';

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
          <strong>Location:</strong> {config.location.path}
        </p>
        <p>
          <strong>Filename encryption:</strong> standard
        </p>
        <p>
          <strong>Encrypt directory names:</strong> true
        </p>
        <p>
          <strong>Safe Key (password):</strong> {config.password}
        </p>
        <p>
          <strong>Salt (password2):</strong> {config.salt}
        </p>
        <p>
          <strong>rclone config:</strong>
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
        {copied ? 'Copied' : 'Copy'}
      </Button>
    </div>
  );
});
