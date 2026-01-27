import { css } from '@emotion/css';
import { memo } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';

import errorIconImage from '../assets/images/error-icon@2x.png';
import { Navbar } from '../components/navbar/Navbar';

export const NotSupportedPage = memo(() => {
  const intl = useIntl();
  const title = intl.formatMessage({
    id: 'web.not_supported_page.title',
    description: 'Document title for the browser not supported page.',
    defaultMessage: 'Not supported',
  });

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
            max-width: 500px;
            text-align: center;
          `}
        >
          <FormattedMessage
            id="web.not_supported_page.description"
            description="Message shown when the current browser is not supported."
            defaultMessage="Your browser is not supported. Please open this page in a modern browser on a computer."
          />
        </h2>
      </div>
    </>
  );
});
NotSupportedPage.displayName = 'NotSupportedPage';
