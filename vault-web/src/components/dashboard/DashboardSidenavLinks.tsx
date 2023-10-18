import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';
import { Link } from 'react-router-dom';

import { buttonReset } from '../../styles/mixins/buttons';
import { allStates } from '../../styles/mixins/hover';
import { useWebVault } from '../../webVault/useWebVault';

import { GitRelease } from '../GitRelease';
import { GitRevision } from '../GitRevision';
import { IntroModalLazy } from '../intro/IntroModalLazy';
import { useIntro } from '../intro/useIntro';

export const DashboardSidenavLinks = memo(() => {
  const theme = useTheme();
  const webVault = useWebVault();
  const baseUrl = webVault.configGetBaseUrl();
  const intro = useIntro();

  return (
    <>
      <footer
        className={css`
          margin: 0 0 0 25px;
        `}
      >
        <div
          className={css`
            margin: 0 0 15px;
            font-size: 12px;
            color: ${theme.colors.text};
            text-align: center;
          `}
        >
          <a
            href="https://koofr.eu/help/koofr-vault/"
            target="_blank"
            rel="noreferrer"
            className={css`
              ${allStates} {
                color: ${theme.colors.text};
              }
            `}
          >
            Help and support
          </a>
          <span aria-hidden>{' · '}</span>
          <button
            type="button"
            className={css`
              ${buttonReset}
              color: ${theme.colors.text};
            `}
            onClick={() => {
              intro.show();
            }}
          >
            Intro
          </button>
          <span aria-hidden>{' · '}</span>
          <a
            href={`${baseUrl}/legal`}
            target="_blank"
            rel="noreferrer"
            className={css`
              ${allStates} {
                color: ${theme.colors.text};
              }
            `}
          >
            Legal
          </a>
          <br />
          <Link
            to="/landing"
            className={css`
              ${allStates} {
                color: ${theme.colors.text};
              }
            `}
          >
            Landing page
          </Link>
        </div>

        <div
          className={css`
            font-size: 12px;
            text-align: center;
          `}
        >
          <GitRelease />
          <GitRevision />
        </div>
      </footer>

      <IntroModalLazy isVisible={intro.isVisible} hide={intro.hide} />
    </>
  );
});
