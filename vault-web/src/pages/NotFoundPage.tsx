import { css } from '@emotion/css';
import { memo } from 'react';

import errorIconImage from '../assets/images/error-icon@2x.png';
import { LinkButton } from '../components/Button';
import { Navbar } from '../components/navbar/Navbar';
import { useDocumentTitle } from '../utils/useDocumentTitle';

export const NotFoundPage = memo(() => {
  useDocumentTitle('Page not found');

  return (
    <>
      <Navbar
        header={
          <span
            className={css`
              font-weight: 600;
            `}
          >
            Page not found
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
          alt=""
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
          Page not found
        </h2>

        <LinkButton to="/" variant="primary">
          Go to dashboard
        </LinkButton>
      </div>
    </>
  );
});
