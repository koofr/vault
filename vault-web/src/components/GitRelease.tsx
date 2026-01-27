import { css } from '@emotion/css';
import { memo } from 'react';
import { FormattedMessage } from 'react-intl';

import { allStates } from '../styles/mixins/hover';

export const GitRelease = memo<{ className?: string }>(({ className }) => {
  // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
  const gitRelease = import.meta.env.VITE_GIT_RELEASE;

  if (gitRelease == null || gitRelease === '') {
    return null;
  }

  return (
    <div className={className}>
      <FormattedMessage
        id="web.version.text"
        description="Footer label showing the app release version with a link to the GitHub release."
        defaultMessage="Version: {version}"
        values={{
          version: (
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
          ),
        }}
      />
    </div>
  );
});
GitRelease.displayName = 'GitRelease';
