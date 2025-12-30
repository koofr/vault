import { css } from '@emotion/css';
import { memo } from 'react';
import { allStates } from '../styles/mixins/hover';

export const GitRelease = memo<{ className?: string }>(({ className }) => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const gitRelease = import.meta.env.VITE_GIT_RELEASE;

  if (gitRelease == null || gitRelease === '') {
    return null;
  }

  return (
    <div className={className}>
      Version:{' '}
      <a
        href={`https://github.com/koofr/vault/releases/tag/${gitRelease}`}
        target="_blank"
        rel="noreferrer"
        className={css`
          ${allStates} {
            color: inherit;
          }
        `}
      >
        {gitRelease}
      </a>
    </div>
  );
});
GitRelease.displayName = 'GitRelease';
