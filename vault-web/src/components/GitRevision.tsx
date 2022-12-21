import { css } from '@emotion/css';
import { memo } from 'react';
import { allStates } from '../styles/mixins/hover';

export const GitRevision = memo<{ className?: string }>(({ className }) => {
  const gitRevision = import.meta.env.VITE_GIT_REVISION;

  return (
    <div className={className}>
      Git revision:{' '}
      <a
        href={`https://github.com/koofr/vault/commit/${gitRevision}`}
        target="_blank"
        rel="noreferrer"
        className={css`
          ${allStates} {
            color: inherit;
          }
        `}
      >
        {gitRevision}
      </a>
    </div>
  );
});
