import { css } from '@emotion/css';
import { memo } from 'react';
import { FormattedMessage } from 'react-intl';

import { allStates } from '../styles/mixins/hover';

export const GitRevision = memo<{ className?: string }>(({ className }) => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const gitRevision = import.meta.env.VITE_GIT_REVISION;

  if (gitRevision == null || gitRevision === '') {
    return null;
  }

  return (
    <div className={className}>
      <FormattedMessage
        id="web.git_revision.text"
        description="Footer label showing the git commit hash with a link to the commit on GitHub."
        defaultMessage="Git revision: {git_revision}"
        values={{
          git_revision: (
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
          ),
        }}
      />
    </div>
  );
});
GitRevision.displayName = 'GitRevision';
