import { css } from '@emotion/css';
import { memo } from 'react';
import { allStates } from '../styles/mixins/hover';

export const GitRevision = memo<{ className?: string }>(({ className }) => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const gitRevision = import.meta.env.VITE_GIT_REVISION;

  if (gitRevision == null || gitRevision === '') {
    return null;
  }

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
GitRevision.displayName = 'GitRevision';
