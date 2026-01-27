import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { Suspense, memo } from 'react';
import { FormattedMessage } from 'react-intl';
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
            margin: 0 0 5px;
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
            <FormattedMessage
              id="web.dashboard_sidenav_links.help_and_support.link"
              description="Footer link label to Koofr Vault help and support."
              defaultMessage="Help and support"
            />
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
            <FormattedMessage
              id="web.dashboard_sidenav_links.intro.link"
              description="Footer button label to open the intro walkthrough modal."
              defaultMessage="Intro"
            />
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
            <FormattedMessage
              id="web.dashboard_sidenav_links.legal.link"
              description="Footer link label to the legal page."
              defaultMessage="Legal"
            />
          </a>
          {import.meta.env.VITE_VAULT_APP === 'desktop' ? null : (
            <>
              <br />
              <Link
                to="/landing"
                className={css`
                  ${allStates} {
                    color: ${theme.colors.text};
                  }
                `}
              >
                <FormattedMessage
                  id="web.dashboard_sidenav_links.landing_page.link"
                  description="Footer link label to the landing page."
                  defaultMessage="Landing page"
                />
              </Link>
            </>
          )}
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

      <Suspense>
        <IntroModalLazy isVisible={intro.isVisible} hide={intro.hide} />
      </Suspense>
    </>
  );
});
DashboardSidenavLinks.displayName = 'DashboardSidenavLinks';
