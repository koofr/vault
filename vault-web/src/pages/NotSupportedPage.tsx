import { css } from '@emotion/css';
import { memo } from 'react';

import errorIconImage from '../assets/images/error-icon@2x.png';
import { Navbar } from '../components/navbar/Navbar';

export const NotSupportedPage = memo(() => {
  return (
    <>
      <Navbar
        header={
          <span
            className={css`
              font-weight: 600;
            `}
          >
            Not supported
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
            max-width: 500px;
            text-align: center;
          `}
        >
          Your browser is not supported. Please open this page in a modern
          browser on a computer.
        </h2>
      </div>
    </>
  );
});
