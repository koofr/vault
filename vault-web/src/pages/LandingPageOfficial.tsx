/* eslint-disable react/jsx-no-target-blank */
import { css, cx } from '@emotion/css';
import { memo } from 'react';
import Typewriter from 'typewriter-effect';

import arrowDownImage from '../assets/images/landing/arrow-down.png';
import arrowDown2xImage from '../assets/images/landing/arrow-down@2x.png';
import arrowRightImage from '../assets/images/landing/arrow-right.png';
import arrowRight2xImage from '../assets/images/landing/arrow-right@2x.png';
import googlePlayImage from '../assets/images/apps/google-play.png';
import googlePlay2xImage from '../assets/images/apps/google-play@2x.png';
import appStoreImage from '../assets/images/apps/app-store.png';
import appStore2xImage from '../assets/images/apps/app-store@2x.png';
import graphic1Image from '../assets/images/landing/graphic-1.png';
import graphic12xImage from '../assets/images/landing/graphic-1@2x.png';
import graphic2Image from '../assets/images/landing/graphic-2.png';
import graphic22xImage from '../assets/images/landing/graphic-2@2x.png';
import graphic3Image from '../assets/images/landing/graphic-3.png';
import graphic32xImage from '../assets/images/landing/graphic-3@2x.png';
import graphic4Image from '../assets/images/landing/graphic-4.png';
import graphic42xImage from '../assets/images/landing/graphic-4@2x.png';
import LogoImage from '../assets/images/landing/logo.svg?react';
import mainGraphicImage from '../assets/images/landing/main-graphic.png';
import mainGraphic2xImage from '../assets/images/landing/main-graphic@2x.png';
import openSourceImage from '../assets/images/landing/open-source.png';
import openSource2xImage from '../assets/images/landing/open-source@2x.png';
import rcloneImage from '../assets/images/landing/rclone.png';
import rclone2xImage from '../assets/images/landing/rclone@2x.png';
import step1Image from '../assets/images/landing/step-1.png';
import step12xImage from '../assets/images/landing/step-1@2x.png';
import step2Image from '../assets/images/landing/step-2.png';
import step22xImage from '../assets/images/landing/step-2@2x.png';
import step3Image from '../assets/images/landing/step-3.png';
import step32xImage from '../assets/images/landing/step-3@2x.png';
import vaultImage from '../assets/images/landing/vault.png';
import vault2xImage from '../assets/images/landing/vault@2x.png';
import { BaseAnchorButton } from '../components/Button';
import { GitRelease } from '../components/GitRelease';
import { GitRevision } from '../components/GitRevision';
import { RetinaImage } from '../components/RetinaImage';
import { buttonStyle } from '../styles/mixins/buttons';
import { allStates } from '../styles/mixins/hover';
import { useDocumentTitle } from '../utils/useDocumentTitle';
import { useConfig } from '../config';

const landingButtonStyle = buttonStyle(
  '#1683fb',
  '#1683fb',
  '#ffffff',
  '#0576f1',
  '#0576f1',
  '#ffffff',
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

const TypingText = memo(() => {
  const keywords = ['private files.', 'confidentials.', 'secrets.'];

  const typewriterOptions = {
    strings: keywords,
    autoStart: true,
    loop: true,
    delay: 70,
    deleteSpeed: 50,
    pauseFor: 1500,
  };

  return (
    <span
      className={css`
        & .Typewriter {
          display: inline;
        }

        & .Typewriter__cursor {
          color: #ffd15c;
          font-weight: 300;
          margin-left: -5px;
          position: relative;
          bottom: 9px;
          font-size: inherit;
        }
      `}
    >
      <Typewriter options={typewriterOptions} />
    </span>
  );
});

export const LandingPageOfficial = memo(() => {
  useDocumentTitle();

  const config = useConfig();

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
        align-items: center;
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
          justify-content: space-between;
          margin-bottom: 30px;

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

        <div
          className={css`
            display: flex;
            align-items: center;
          `}
        >
          <BaseAnchorButton
            href="/login"
            className={cx(
              landingButtonStyle,
              css`
                font-size: 16px;
                padding: 12px 19px;
                font-weight: 700;
                width: 131px;
                line-height: 22px;

                ${bp.sm} {
                  font-size: 14px;
                  padding: 7px 6px;
                  width: inherit;
                  line-height: 22px;
                  width: 103px;
                }

                ${bp.md} {
                  line-height: 12px;
                }
              `,
            )}
          >
            Get started
          </BaseAnchorButton>
        </div>
      </div>

      <div
        className={css`
          display: flex;
          flex-direction: row;
          margin-bottom: 41px;

          ${bp.sm} {
            padding-left: 15px;
            padding-right: 15px;
          }

          ${bp.smmd} {
            flex-direction: column;
            max-width: 556px;
          }

          ${bp.mdlg} {
            padding-left: 28px;
            padding-right: 28px;
          }

          ${bp.lg} {
            width: 100%;
          }

          ${bp.xl} {
            width: 1280px;
          }
        `}
      >
        <div
          className={css`
            display: flex;
            flex-direction: column;
            flex-grow: 1;
          `}
        >
          <h1
            className={css`
              font-size: 64px;
              line-height: 1.08;
              font-weight: 700;
              color: #011722;
              margin: 0 0 36px;

              ${bp.sm} {
                font-size: 51px;
                margin: 0 0 26px;
              }

              ${bp.md} {
                width: 700px;
              }

              @media (max-width: 548px) {
                font-size: 45px;
              }

              @media (max-width: 482px) {
                font-size: 43px;
              }

              @media (max-width: 450px) {
                font-size: 38px;
              }

              @media (max-width: 420px) {
                font-size: 28px;
              }
            `}
          >
            One vault for all
            <br />
            <span
              className={css`
                font-weight: 800;
              `}
            >
              your <TypingText />
            </span>
          </h1>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 43px;

              ${bp.sm} {
                font-size: 14px;
                margin: 0 0 26px;
              }

              ${bp.lgxl} {
                width: 565px;
              }
            `}
          >
            Powerful, open source, client-side, zero-knowledge encryption.
            Unlock enhanced security for your most sensitive files with Koofr
            Vault.
          </p>

          <BaseAnchorButton
            href="/login"
            className={cx(
              landingButtonStyle,
              css`
                font-size: 18px;
                padding: 9px 21px;
                font-weight: 700;

                ${bp.smmd} {
                  width: 100%;
                }

                ${bp.lgxl} {
                  width: 500px;
                }
              `,
            )}
          >
            Get started
          </BaseAnchorButton>

          {config.appStoreUrl !== undefined ||
          config.googlePlayUrl !== undefined ? (
            <div
              className={css`
                display: flex;
                flex-direction: row;
                margin-top: 32px;
              `}
            >
              {config.googlePlayUrl !== undefined ? (
                <a
                  href={config.googlePlayUrl}
                  target="_blank"
                  rel="noreferrer"
                  className={css`
                    margin-right: 20px;
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
                <a href={config.appStoreUrl} target="_blank" rel="noreferrer">
                  <RetinaImage
                    image={appStoreImage}
                    image2x={appStore2xImage}
                    width={117}
                    height={36}
                  />
                </a>
              ) : null}
            </div>
          ) : null}
        </div>

        <div
          className={css`
            ${bp.smmd} {
              display: flex;
              flex-direction: column;
              align-items: center;
              padding: 38px 0;
            }

            ${bp.lgxl} {
              padding: 25px 33px 0;
              flex-shrink: 0;
            }
          `}
        >
          <RetinaImage
            image={mainGraphicImage}
            image2x={mainGraphic2xImage}
            width={480}
            height={338}
            classNameForSize={(width, height) => css`
              ${bp.sm} {
                background-size: 290px ${(290 * height) / width}px;
                width: 290px;
                height: ${(290 * height) / width}px;
              }

              @media (min-width: 1000px) and (max-width: 1049px) {
                background-size: ${width * 0.6}px ${height * 0.6}px;
                width: ${width * 0.6}px;
                height: ${height * 0.6}px;
              }

              @media (min-width: 1050px) and (max-width: 1169px) {
                background-size: ${width * 0.7}px ${height * 0.7}px;
                width: ${width * 0.7}px;
                height: ${height * 0.7}px;
              }
              @media (min-width: 1170px) and (max-width: 1271px) {
                background-size: ${width * 0.8}px ${height * 0.8}px;
                width: ${width * 0.8}px;
                height: ${height * 0.8}px;
              }
            `}
          />
        </div>
      </div>

      <div
        className={css`
          display: flex;
          flex-direction: column;
          margin-bottom: 107px;

          ${bp.sm} {
            padding-left: 15px;
            padding-right: 15px;
          }

          ${bp.smmd} {
            max-width: 556px;
          }

          ${bp.mdlg} {
            padding-left: 28px;
            padding-right: 28px;
          }

          ${bp.lg} {
            width: 100%;
          }

          ${bp.xl} {
            width: 1280px;
          }
        `}
      >
        <h2
          className={css`
            font-size: 30px;
            font-weight: 700;
            color: #011722;
            margin: 0 0 30px;
          `}
        >
          Extra strong protection
        </h2>

        <p
          className={css`
            font-size: 18px;
            color: #011722;

            ${bp.smmd} {
              margin: 0 0 50px;
            }

            ${bp.lgxl} {
              width: 500px;
              margin: 0 0 32px;
            }
          `}
        >
          Koofr Vault is an open source, client-side, zero-knowledge encrypted
          storage application by{' '}
          <a href="https://koofr.eu" target="_blank" rel="noopener">
            Koofr cloud storage
          </a>
          . It's like having a box with a unique lock in your trunk: an extra
          layer of security to protect your files.
        </p>

        <div
          className={css`
            display: flex;

            ${bp.smmd} {
              flex-direction: column;
              align-items: center;
            }

            ${bp.lgxl} {
              flex-direction: row;
              justify-content: space-between;
            }
          `}
        >
          <div
            className={css`
              display: flex;
              flex-direction: column;
              width: 286px;

              @media (min-width: 1000px) and (max-width: 1069px) {
                width: 250px;
              }

              @media (min-width: 1070px) and (max-width: 1170px) {
                width: 250px;
              }
            `}
          >
            <div
              className={css`
                height: 150px;
                border: 1px solid #d4d6d7;
                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: center;
                margin-bottom: 22px;
              `}
            >
              <RetinaImage
                image={step1Image}
                image2x={step12xImage}
                width={70}
                height={86}
              />
            </div>
            <p
              className={css`
                font-size: 18px;
                color: #011722;
                margin: 0;
              `}
            >
              Choose a special Safe Key to create a Safe Box in your Koofr
              Vault.
            </p>
          </div>

          <div
            className={css`
              height: 152px;
              display: flex;
              flex-direction: column;
              justify-content: center;
              align-items: center;
            `}
          >
            <div
              className={css`
                ${bp.lgxl} {
                  display: none;
                }
              `}
            >
              <RetinaImage
                image={arrowDownImage}
                image2x={arrowDown2xImage}
                width={18}
                height={74}
              />
            </div>
            <div
              className={css`
                ${bp.smmd} {
                  display: none;
                }
              `}
            >
              <RetinaImage
                image={arrowRightImage}
                image2x={arrowRight2xImage}
                width={74}
                height={18}
              />
            </div>
          </div>

          <div
            className={css`
              display: flex;
              flex-direction: column;
              width: 286px;

              @media (min-width: 1000px) and (max-width: 1069px) {
                width: 250px;
              }

              @media (min-width: 1070px) and (max-width: 1170px) {
                width: 250px;
              }
            `}
          >
            <div
              className={css`
                height: 150px;
                border: 1px solid #d4d6d7;
                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: center;
                margin-bottom: 22px;
              `}
            >
              <RetinaImage
                image={step2Image}
                image2x={step22xImage}
                width={70}
                height={86}
              />
            </div>
            <p
              className={css`
                font-size: 18px;
                color: #011722;
                margin: 0;
              `}
            >
              Your Safe Box files are encrypted or decrypted by your device on
              demand as you access them.
            </p>
          </div>

          <div
            className={css`
              height: 152px;
              display: flex;
              flex-direction: column;
              justify-content: center;
              align-items: center;
            `}
          >
            <div
              className={css`
                ${bp.lgxl} {
                  display: none;
                }
              `}
            >
              <RetinaImage
                image={arrowDownImage}
                image2x={arrowDown2xImage}
                width={18}
                height={74}
              />
            </div>
            <div
              className={css`
                ${bp.smmd} {
                  display: none;
                }
              `}
            >
              <RetinaImage
                image={arrowRightImage}
                image2x={arrowRight2xImage}
                width={74}
                height={18}
              />
            </div>
          </div>

          <div
            className={css`
              display: flex;
              flex-direction: column;
              width: 286px;

              @media (min-width: ${bpDim.lgMinWidth}px) and (max-width: 1069px) {
                width: 210px;
              }

              @media (min-width: 1070px) and (max-width: 1170px) {
                width: 250px;
              }
            `}
          >
            <div
              className={css`
                height: 150px;
                border: 1px solid #d4d6d7;
                display: flex;
                flex-direction: column;
                justify-content: center;
                align-items: center;
                margin-bottom: 22px;
              `}
            >
              <RetinaImage
                image={step3Image}
                image2x={step32xImage}
                width={70}
                height={86}
              />
            </div>
            <p
              className={css`
                font-size: 18px;
                color: #011722;
                margin: 0;
              `}
            >
              Cloud content is always encrypted and never accessible without
              your Safe Key.
            </p>
          </div>
        </div>
      </div>

      <div
        className={css`
          width: 100%;
          display: flex;

          ${bp.sm} {
            padding-left: 15px;
            padding-right: 15px;
          }

          ${bp.smmd} {
            flex-direction: column-reverse;
            align-items: center;
            margin-bottom: 120px;
          }

          ${bp.mdlg} {
            padding-left: 28px;
            padding-right: 28px;
          }

          ${bp.lgxl} {
            flex-direction: row;
            align-items: center;
            margin-bottom: 120px;
          }

          ${bp.xl} {
            width: 1280px;
          }
        `}
      >
        <div
          className={css`
            width: 60%;
            display: flex;
            flex-direction: column;
            align-items: center;
            flex-grow: 1;
          `}
        >
          <RetinaImage
            image={graphic4Image}
            image2x={graphic42xImage}
            width={356}
            height={414}
            classNameForSize={(width, height) => css`
              ${bp.sm} {
                background-size: 290px ${(290 * height) / width}px;
                width: 290px;
                height: ${(290 * height) / width}px;
              }
            `}
          />
        </div>

        <div
          className={css`
            display: flex;
            flex-direction: column;
            max-width: 500px;
            flex-shrink: 0;
          `}
        >
          <h2
            className={css`
              font-size: 30px;
              font-weight: 700;
              color: #011722;
              margin: 0 0 22px;
            `}
          >
            For your eyes only
          </h2>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 32px;
            `}
          >
            Your files are encrypted, file names and all content included,
            locally on your device with your Safe Key and some magic salt before
            they are sent to your Vault.
          </p>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 32px;

              ${bp.smmd} {
                margin: 0 0 70px;
              }
            `}
          >
            Neither your Safe key nor any other unencrypted file data or
            metadata is sent to or stored by{' '}
            <a href="https://koofr.eu" target="_blank" rel="noopener">
              Koofr
            </a>
            . Only you can decrypt and access your Vault files.
          </p>
        </div>
      </div>

      <div
        className={css`
          width: 100%;
          display: flex;
          flex-direction: column;
          align-items: center;
          background-color: #011722;

          ${bp.smmd} {
            padding: 60px 0 60px;
          }

          ${bp.lgxl} {
            padding: 111px 0 120px;
          }
        `}
      >
        <div
          className={css`
            display: flex;

            ${bp.sm} {
              padding-left: 15px;
              padding-right: 15px;
            }

            ${bp.smmd} {
              flex-direction: column;
              max-width: 556px;
            }

            ${bp.mdlg} {
              padding-left: 28px;
              padding-right: 28px;
            }

            ${bp.lg} {
              width: 100%;
            }

            ${bp.lgxl} {
              flex-direction: row;
              justify-content: space-between;
            }

            ${bp.xl} {
              width: 1280px;
            }
          `}
        >
          <div
            className={css`
              display: flex;
              flex-direction: column;
              flex-grow: 1;

              ${bp.smmd} {
                align-items: center;
                margin-bottom: 40px;
              }

              ${bp.lgxl} {
                margin-top: 71px;
              }

              @media (min-width: ${bpDim.lgMinWidth}px) and (max-width: 1249px) {
                display: none;
              }
            `}
          >
            <a
              href="https://github.com/koofr/vault"
              target="_blank"
              rel="noreferrer"
            >
              <RetinaImage
                image={openSourceImage}
                image2x={openSource2xImage}
                width={308}
                height={150}
              />
            </a>
          </div>

          <div
            className={css`
              display: flex;
              flex-direction: column;
              flex-shrink: 0;

              ${bp.lgxl} {
                width: 500px;
                margin: 0 40px;
              }
            `}
          >
            <h2
              className={css`
                font-size: 30px;
                font-weight: 700;
                color: #f4f5f5;
                margin: 0 0 22px;
              `}
            >
              Fully open source
            </h2>

            <p
              className={css`
                font-size: 18px;
                color: #f4f5f5;
                margin: 0 0 32px;
              `}
            >
              Koofr Vault is{' '}
              <a
                href="https://github.com/koofr/vault"
                target="_blank"
                rel="noreferrer"
                className={css`
                  ${allStates} {
                    color: #f4f5f5;
                    text-decoration: underline;
                  }
                `}
              >
                open source
              </a>
              , so you can always check that the code does exactly what is
              promised - and nothing more. File encryption is performed using
              NaCl SecretBox, which uses XSalsa20 cipher and Poly1305 for
              ensuring integrity.
            </p>

            <p
              className={css`
                font-size: 18px;
                color: #f4f5f5;

                ${bp.smmd} {
                  margin: 0 0 50px;
                }

                ${bp.lgxl} {
                  margin: 0 0 78px;
                }

                @media (min-width: ${bpDim.lgMinWidth}px) and (max-width: 1249px) {
                  margin: 0 0 0;
                }
              `}
            >
              It is compatible with{' '}
              <a
                href="https://rclone.org"
                target="_blank"
                rel="noreferrer"
                className={css`
                  ${allStates} {
                    color: #f4f5f5;
                    text-decoration: underline;
                  }
                `}
              >
                rclone
              </a>
              . This means that you can download your encrypted files and
              decrypt them locally using the rclone command-line tool.
            </p>
          </div>

          <div
            className={css`
              display: flex;
              flex-direction: column;

              ${bp.smmd} {
                align-items: center;
              }

              ${bp.lgxl} {
                justify-content: flex-end;
                align-items: flex-end;
                flex-grow: 1;
              }

              @media (min-width: ${bpDim.lgMinWidth}px) and (max-width: 1249px) {
                align-items: center;
              }
            `}
          >
            <div
              className={css`
                display: none;

                @media (min-width: ${bpDim.lgMinWidth}px) and (max-width: 1249px) {
                  display: flex;
                  margin: 0 22px 30px 0;
                }
              `}
            >
              <a
                href="https://github.com/koofr/vault"
                target="_blank"
                rel="noreferrer"
              >
                <RetinaImage
                  image={openSourceImage}
                  image2x={openSource2xImage}
                  width={308}
                  height={150}
                />
              </a>
            </div>

            <div
              className={css`
                display: flex;
                flex-direction: row;
                justify-content: center;
                align-items: center;
                border: 1px solid #565656;
                width: 288px;
                height: 152px;
              `}
            >
              <a
                href="https://github.com/koofr/vault"
                target="_blank"
                rel="noreferrer"
                className={css`
                  margin: 0 23px;
                `}
              >
                <RetinaImage
                  image={vaultImage}
                  image2x={vault2xImage}
                  width={54}
                  height={54}
                />
              </a>
              <a
                href="https://rclone.org/"
                target="_blank"
                rel="noreferrer"
                className={css`
                  margin: 0 23px;
                `}
              >
                <RetinaImage
                  image={rcloneImage}
                  image2x={rclone2xImage}
                  width={55}
                  height={52}
                />
              </a>
            </div>
          </div>
        </div>
      </div>

      <div
        className={css`
          display: flex;

          ${bp.sm} {
            padding-left: 15px;
            padding-right: 15px;
          }

          ${bp.smmd} {
            flex-direction: column;
            max-width: 556px;
            padding-top: 60px;
            padding-bottom: 60px;
          }

          ${bp.mdlg} {
            padding-left: 28px;
            padding-right: 28px;
          }

          ${bp.lg} {
            width: 100%;
          }

          ${bp.lgxl} {
            flex-direction: row;
            justify-content: center;
            padding-top: 76px;
            padding-bottom: 76px;
          }

          ${bp.xl} {
            width: 1280px;
          }
        `}
      >
        <div
          className={css`
            display: flex;
            flex-direction: column;
            align-items: center;

            ${bp.smmd} {
              margin-bottom: 50px;
            }

            ${bp.lgxl} {
              width: 350px;
            }
          `}
        >
          <div
            className={css`
              width: 150px;
              height: 112px;
              border: 1px solid #d4d6d7;
              display: flex;
              flex-direction: column;
              justify-content: center;
              align-items: center;
              margin-bottom: 15px;
            `}
          >
            <RetinaImage
              image={graphic1Image}
              image2x={graphic12xImage}
              width={70}
              height={61}
            />
          </div>
          <p
            className={css`
              font-size: 18px;
              font-weight: 600;
              color: #011722;
              margin: 0;
              text-align: center;
            `}
          >
            Client-side encryption
          </p>
        </div>

        <div
          className={css`
            display: flex;
            flex-direction: column;
            align-items: center;

            ${bp.smmd} {
              margin-bottom: 50px;
            }

            ${bp.lgxl} {
              width: 350px;
            }
          `}
        >
          <div
            className={css`
              width: 150px;
              height: 112px;
              border: 1px solid #d4d6d7;
              display: flex;
              flex-direction: column;
              justify-content: center;
              align-items: center;
              margin-bottom: 15px;
            `}
          >
            <RetinaImage
              image={graphic2Image}
              image2x={graphic22xImage}
              width={70}
              height={61}
            />
          </div>
          <p
            className={css`
              font-size: 18px;
              font-weight: 600;
              color: #011722;
              margin: 0;
              text-align: center;
            `}
          >
            Verifiable source code
          </p>
        </div>

        <div
          className={css`
            display: flex;
            flex-direction: column;
            align-items: center;

            ${bp.smmd} {
              margin-bottom: 0;
            }

            ${bp.lgxl} {
              width: 350px;
            }
          `}
        >
          <div
            className={css`
              width: 150px;
              height: 112px;
              border: 1px solid #d4d6d7;
              display: flex;
              flex-direction: column;
              justify-content: center;
              align-items: center;
              margin-bottom: 15px;
            `}
          >
            <RetinaImage
              image={graphic3Image}
              image2x={graphic32xImage}
              width={70}
              height={61}
            />
          </div>
          <p
            className={css`
              font-size: 18px;
              font-weight: 600;
              color: #011722;
              margin: 0;
              text-align: center;
            `}
          >
            Zero knowledge
          </p>
        </div>
      </div>

      <div
        className={css`
          width: 100%;
          border-bottom: 1px solid #d4d6d7;

          ${bp.smmd} {
            margin-bottom: 60px;
          }

          ${bp.lgxl} {
            margin-bottom: 111px;
          }
        `}
      ></div>

      <div
        className={css`
          display: flex;
          flex-direction: column;
          align-items: center;

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
          <h2
            className={css`
              font-size: 30px;
              font-weight: 700;
              color: #011722;
              flex-grow: 1;
              margin: 0 0 22px;
              text-align: center;
            `}
          >
            Unlock your Vault
          </h2>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 43px;
              text-align: center;
            `}
          >
            Start encrypting your cloud storage files in just a few minutes.
            Included in all{' '}
            <a href="https://koofr.eu/pricing" target="_blank" rel="noopener">
              Koofr plans
            </a>
            .
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
              `,
            )}
          >
            Get started
          </BaseAnchorButton>
        </div>
        <p
          className={css`
            font-size: 18px;
            color: #011722;
            margin: 43px 0 0;
            text-align: center;
          `}
        >
          Have questions about Koofr Vault?{' '}
          <a
            href="https://koofr.eu/help/koofr-vault"
            target="_blank"
            rel="noopener"
          >
            Find answers here
          </a>
          .
        </p>
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
