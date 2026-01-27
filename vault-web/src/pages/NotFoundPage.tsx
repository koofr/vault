import { css } from '@emotion/css';
import { memo } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

import errorIconImage from '../assets/images/error-icon@2x.png';
import { LinkButton } from '../components/Button';
import { Navbar } from '../components/navbar/Navbar';
import { useDocumentTitle } from '../utils/useDocumentTitle';

export const NotFoundPage = memo(() => {
  const intl = useIntl();
  const title = intl.formatMessage({
    id: 'web.not_found_page.title',
    description: 'Document title and main heading for the 404 page.',
    defaultMessage: 'Page not found',
  });
  useDocumentTitle(title);

  return (
    <>
      <Navbar
        header={
          <span
            className={css`
              font-weight: 600;
            `}
          >
            {title}
          </span>
        }
      />
      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;
        `}
      >
        <img
          src={errorIconImage}
          alt={title}
          className={css`
            display: block;
            width: 252px;
            height: 186px;
            margin: 0 0 30px;
          `}
        />
        <h2
          className={css`
            font-size: 32px;
            font-weight: normal;
            margin: 0 0 30px;
          `}
        >
          {title}
        </h2>

        <LinkButton to="/" variant="primary">
          <FormattedMessage
            id="web.not_found_page.go_to_dashboard.button"
            description="Primary button on the 404 page that returns the user to the dashboard."
            defaultMessage="Go to dashboard"
          />
        </LinkButton>
      </div>
    </>
  );
});
NotFoundPage.displayName = 'NotFoundPage';
