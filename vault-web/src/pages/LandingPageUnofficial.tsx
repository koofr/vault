/* eslint-disable react/jsx-no-target-blank */
import { css, cx } from '@emotion/css';
import { memo } from 'react';

import { ReactComponent as LogoImage } from '../assets/images/landing/logo.svg';
import { BaseAnchorButton } from '../components/Button';
import { GitRelease } from '../components/GitRelease';
import { GitRevision } from '../components/GitRevision';
import { buttonStyle } from '../styles/mixins/buttons';
import { allStates } from '../styles/mixins/hover';
import { useDocumentTitle } from '../utils/useDocumentTitle';

const landingButtonStyle = buttonStyle(
  '#1683fb',
  '#1683fb',
  '#ffffff',
  '#0576f1',
  '#0576f1',
  '#ffffff'
);

const bpDim = {
  smMaxWidth: 767,
  mdMinWidth: 768,
  mdMaxWidth: 1048,
  lgMinWidth: 1049,
  lgMaxWidth: 1365,
  xlMinWidth: 1366,
};

const bp = {
  sm: `@media (max-width: ${bpDim.smMaxWidth}px)`,
  smmd: `@media (max-width: ${bpDim.mdMaxWidth}px)`,
  md: `@media (min-width: ${bpDim.mdMinWidth}px) and (max-width: ${bpDim.mdMaxWidth}px)`,
  mdlg: `@media (min-width: ${bpDim.mdMinWidth}px) and (max-width: ${bpDim.lgMaxWidth}px)`,
  lg: `@media (min-width: ${bpDim.lgMinWidth}px) and (max-width: ${bpDim.lgMaxWidth}px)`,
  lgxl: `@media (min-width: ${bpDim.lgMinWidth}px)`,
  xl: `@media (min-width: ${bpDim.xlMinWidth}px)`,
};

export const LandingPageUnofficial = memo(() => {
  useDocumentTitle();

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
        align-items: center;
        min-height: 100vh;
      `}
    >
      <div
        className={css`
          width: 100%;
          padding-top: 20px;
          padding-bottom: 20px;
          display: flex;
          flex-direction: row;
          align-items: center;
          justify-content: center;
          margin-bottom: 30px;
          flex-shrink: 0;

          ${bp.sm} {
            padding-left: 15px;
            padding-right: 15px;
            padding-top: 7px;
            padding-bottom: 15px;
          }

          ${bp.md} {
            padding-top: 20px;
          }

          ${bp.mdlg} {
            padding-left: 28px;
            padding-right: 28px;
          }

          ${bp.xl} {
            width: 1280px;
          }
        `}
      >
        <LogoImage />
      </div>

      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;
          flex-grow: 1;

          ${bp.sm} {
            padding-left: 15px;
            padding-right: 15px;
          }

          ${bp.smmd} {
            margin: 0 0 60px;
          }

          ${bp.lgxl} {
            margin: 0 0 111px;
          }
        `}
      >
        <div
          className={css`
            display: flex;
            flex-direction: column;
            max-width: 500px;
          `}
        >
          <h1
            className={css`
              font-size: 64px;
              font-weight: 700;
              color: #011722;
              flex-grow: 1;
              margin: 0 0 22px;
              text-align: center;
            `}
          >
            Koofr Vault
          </h1>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 43px;
              text-align: center;
            `}
          >
            This is an unofficial Koofr Vault page. Use at your own risk.
          </p>

          <BaseAnchorButton
            href="/login"
            className={cx(
              landingButtonStyle,
              css`
                width: 100%;
                font-size: 18px;
                padding: 9px 21px;
                font-weight: 700;
              `
            )}
          >
            Get started
          </BaseAnchorButton>
        </div>
      </div>

      <div
        className={css`
          width: 100%;
          border-bottom: 1px solid #d4d6d7;
        `}
      ></div>

      <div
        className={css`
          ${bp.smmd} {
            padding: 30px 28px;
          }

          ${bp.lgxl} {
            padding: 40px 28px;
          }
        `}
      >
        <p
          className={css`
            font-size: 14px;
            color: #011722;
            text-align: center;
            margin: 0 0 15px;
          `}
        >
          © 2023.{' '}
          <a
            href="https://koofr.eu"
            target="_blank"
            rel="noopener"
            className={css`
              font-weight: 600;
              ${allStates} {
                color: #011722;
                text-decoration: none;
              }
            `}
          >
            Koofr d.o.o.
          </a>{' '}
          all rights reserved.
        </p>

        <div
          className={css`
            font-size: 13px;
            color: #011722;
            text-align: center;
          `}
        >
          <GitRelease />
          <GitRevision />
        </div>
      </div>
    </div>
  );
});
