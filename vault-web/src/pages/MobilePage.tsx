import { css } from '@emotion/css';
import { memo } from 'react';
import { useLocation } from 'react-router-dom';

import appStoreImage from '../assets/images/apps/app-store.png';
import appStore2xImage from '../assets/images/apps/app-store@2x.png';
import fDroidImage from '../assets/images/apps/f-droid.png';
import fDroid2xImage from '../assets/images/apps/f-droid@2x.png';
import googlePlayImage from '../assets/images/apps/google-play.png';
import googlePlay2xImage from '../assets/images/apps/google-play@2x.png';
import { LinkButton } from '../components/Button';
import { RetinaImage } from '../components/RetinaImage';
import { Navbar } from '../components/navbar/Navbar';
import { useConfig } from '../config';
import { useDocumentTitle } from '../utils/useDocumentTitle';

export const MobilePage = memo(() => {
  useDocumentTitle('Open in Vault mobile app');
  const config = useConfig();
  const location = useLocation();
  const pathname = location.pathname.replace(/^\/mobile/, '');
  const to = (pathname !== '' ? pathname : '/') + location.search;

  return (
    <>
      <Navbar
        header={
          <span
            className={css`
              font-weight: 600;
            `}
          >
            Open in mobile app
          </span>
        }
      />
      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;
          text-align: center;
          padding: 0 15px;
        `}
      >
        <h2
          className={css`
            font-size: 32px;
            font-weight: normal;
            margin: 40px 0 30px;
            text-align: center;
          `}
        >
          Open in Vault mobile app
        </h2>

        <p
          className={css`
            font-size: 18px;
            font-weight: normal;
            margin: 0 0 15px;
            text-align: center;
          `}
        >
          It looks like you don't have Koofr Vault mobile app installed.
        </p>

        <p
          className={css`
            font-size: 18px;
            font-weight: normal;
            margin: 0 0 15px;
            text-align: center;
          `}
        >
          Install the mobile app
        </p>

        {config.appStoreUrl !== undefined ||
        config.googlePlayUrl !== undefined ||
        config.fDroidUrl !== undefined ? (
          <div
            className={css`
              display: flex;
              flex-direction: column;
              margin: 0 0 20px;
            `}
          >
            {config.googlePlayUrl !== undefined ? (
              <a
                href={config.googlePlayUrl}
                target="_blank"
                rel="noreferrer"
                className={css`
                  margin-bottom: 15px;
                `}
              >
                <RetinaImage
                  image={googlePlayImage}
                  image2x={googlePlay2xImage}
                  width={122}
                  height={36}
                />
              </a>
            ) : null}

            {config.appStoreUrl !== undefined ? (
              <a
                href={config.appStoreUrl}
                target="_blank"
                rel="noreferrer"
                className={css`
                  margin-bottom: 15px;
                `}
              >
                <RetinaImage
                  image={appStoreImage}
                  image2x={appStore2xImage}
                  width={117}
                  height={36}
                />
              </a>
            ) : null}

            {config.fDroidUrl !== undefined ? (
              <a href={config.fDroidUrl} target="_blank" rel="noreferrer">
                <RetinaImage
                  image={fDroidImage}
                  image2x={fDroid2xImage}
                  width={123}
                  height={36}
                />
              </a>
            ) : null}
          </div>
        ) : null}

        <LinkButton to={to} variant="primary">
          Or continue using the web app
        </LinkButton>
      </div>
    </>
  );
});
