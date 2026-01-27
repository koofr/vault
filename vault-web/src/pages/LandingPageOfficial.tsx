/* eslint-disable react/jsx-no-target-blank */
import { css, cx } from '@emotion/css';
import { memo } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';
import Typewriter from 'typewriter-effect';

import appStoreImage from '../assets/images/apps/app-store.png';
import appStore2xImage from '../assets/images/apps/app-store@2x.png';
import fDroidImage from '../assets/images/apps/f-droid.png';
import fDroid2xImage from '../assets/images/apps/f-droid@2x.png';
import googlePlayImage from '../assets/images/apps/google-play.png';
import googlePlay2xImage from '../assets/images/apps/google-play@2x.png';
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
import { LanguagePickerDropdown } from '../components/languagepicker/LanguagePickerDropdown';
import { useConfig } from '../config';
import { buttonStyle } from '../styles/mixins/buttons';
import { allStates } from '../styles/mixins/hover';
import { useDocumentTitle } from '../utils/useDocumentTitle';

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

const TypingText = memo(({ strings }: { strings: string[] }) => {
  const typewriterOptions = {
    strings,
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
TypingText.displayName = 'TypingText';

export const LandingPageOfficial = memo(() => {
  const intl = useIntl();
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
        <div
          className={css`
            flex-shrink: 0;
          `}
        >
          <LogoImage />
        </div>

        <div
          className={css`
            display: flex;
            align-items: center;
            min-width: 0;
          `}
        >
          <div
            className={css`
              margin-right: 25px;
              overflow: hidden;

              ${bp.sm} {
                margin-right: 10px;
                margin-left: 10px;
              }
            `}
          >
            <LanguagePickerDropdown
              size="large"
              placement="bottom"
              dropdownToggleClassName={css`
                width: 100%;
              `}
            />
          </div>

          <BaseAnchorButton
            href="/login"
            className={cx(
              landingButtonStyle,
              css`
                font-size: 16px;
                padding: 12px 19px;
                font-weight: 700;
                line-height: 22px;
                flex-shrink: 0;

                ${bp.sm} {
                  font-size: 14px;
                  padding: 7px 10px;
                  line-height: 22px;
                }

                ${bp.md} {
                  line-height: 12px;
                }
              `,
            )}
          >
            <FormattedMessage
              id="web.landing_page.get_started.button"
              description="Primary call-to-action button on the landing page that starts login."
              defaultMessage="Get started"
            />
          </BaseAnchorButton>
        </div>
      </div>

      <div
        className={css`
          display: flex;
          flex-direction: row;
          margin-bottom: 70px;

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
            <FormattedMessage
              id="web.landing_page.title"
              description="Hero headline on the official landing page with animated keyword."
              defaultMessage="One vault for all<br></br><b>your {typing_text}</b>"
              values={{
                br: () => <br />,
                b: (chunks) => (
                  <span
                    className={css`
                      font-weight: 800;
                    `}
                  >
                    {chunks}
                  </span>
                ),
                typing_text: (
                  <TypingText
                    strings={[
                      intl.formatMessage({
                        id: 'web.landing_page.title.typing_text_1',
                        description:
                          'First animated keyword in the landing page hero headline.',
                        defaultMessage: 'private files.',
                      }),
                      intl.formatMessage({
                        id: 'web.landing_page.title.typing_text_2',
                        description:
                          'Second animated keyword in the landing page hero headline.',
                        defaultMessage: 'confidentials.',
                      }),
                      intl.formatMessage({
                        id: 'web.landing_page.title.typing_text_3',
                        description:
                          'Third animated keyword in the landing page hero headline.',
                        defaultMessage: 'secrets.',
                      }),
                    ]}
                  />
                ),
              }}
            />
          </h1>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 38px;

              ${bp.sm} {
                font-size: 14px;
                margin: 0 0 26px;
              }

              ${bp.lgxl} {
                width: 565px;
              }
            `}
          >
            <FormattedMessage
              id="web.landing_page.description"
              description="Supporting hero paragraph on the official landing page."
              defaultMessage="Powerful, open source, client-side, zero-knowledge encryption. Unlock enhanced security for your most sensitive files with Koofr Vault."
            />
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
            <FormattedMessage
              id="web.landing_page.get_started.button"
              description="Primary call-to-action button on the landing page that starts login."
              defaultMessage="Get started"
            />
          </BaseAnchorButton>

          {config.appStoreUrl !== undefined ||
          config.googlePlayUrl !== undefined ||
          config.fDroidUrl !== undefined ? (
            <div
              className={css`
                display: flex;
                flex-direction: row;
                flex-wrap: wrap;
                gap: 20px;
                margin-top: 32px;

                ${bp.smmd} {
                  justify-content: center;
                }
              `}
            >
              {config.googlePlayUrl !== undefined ? (
                <a href={config.googlePlayUrl} target="_blank" rel="noreferrer">
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
            margin: 0 0 23px;
          `}
        >
          <FormattedMessage
            id="web.landing_page.section_1.title"
            description="Section title describing overall security benefits on the landing page."
            defaultMessage="Extra strong protection"
          />
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
          <FormattedMessage
            id="web.landing_page.section_1.content"
            description="Section paragraph explaining Vault and Koofr with security metaphor."
            defaultMessage="Koofr Vault is an open source, client-side, zero-knowledge encrypted storage application by <a>Koofr cloud storage</a>. It's like having a box with a unique lock in your trunk: an extra layer of security to protect your files."
            values={{
              a: (chunks) => (
                <a href="https://koofr.eu" target="_blank" rel="noopener">
                  {chunks}
                </a>
              ),
            }}
          />
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
            <h3
              className={css`
                font-size: 18px;
                font-weight: 600;
                color: #011722;
                margin: 0 0 10px;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_2.title"
                description="Feature card title about private Safe Key on the landing page."
                defaultMessage="Private Safe Key"
              />
            </h3>
            <p
              className={css`
                font-size: 18px;
                color: #011722;
                margin: 0;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_2.content"
                description="Feature card text explaining choosing a Safe Key."
                defaultMessage="Choose a Safe Key to create a Safe Box in your Koofr Vault."
              />
            </p>
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
            <h3
              className={css`
                font-size: 18px;
                font-weight: 600;
                color: #011722;
                margin: 0 0 10px;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_3.title"
                description="Feature card title about encryption happening in the browser."
                defaultMessage="It all stays in your browser"
              />
            </h3>
            <p
              className={css`
                font-size: 18px;
                color: #011722;
                margin: 0;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_3.content"
                description="Feature card text about on-device encryption/decryption."
                defaultMessage="Your Safe Box files are encrypted or decrypted on demand, on your device."
              />
            </p>
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
            <h3
              className={css`
                font-size: 18px;
                font-weight: 600;
                color: #011722;
                margin: 0 0 10px;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_4.title"
                description="Feature card title about exclusive access to encrypted content."
                defaultMessage="Only you have access"
              />
            </h3>
            <p
              className={css`
                font-size: 18px;
                color: #011722;
                margin: 0;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_4.content"
                description="Feature card text explaining that content isn't accessible without the Safe Key."
                defaultMessage="Cloud content is encrypted and never accessible without your Safe Key."
              />
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
            <FormattedMessage
              id="web.landing_page.section_5.title"
              description="Section title about privacy and exclusive access."
              defaultMessage="For your eyes only"
            />
          </h2>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 32px;
            `}
          >
            <FormattedMessage
              id="web.landing_page.section_5.content_1"
              description="Section paragraph describing local encryption before upload."
              defaultMessage="Your files are encrypted, file names and all content included, locally on your device with your Safe Key and some magic salt before they are sent to your Vault."
            />
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
            <FormattedMessage
              id="web.landing_page.section_5.content_2"
              description="Section paragraph explaining that Koofr never receives unencrypted data."
              defaultMessage="Neither your Safe key nor any other unencrypted file data or metadata is sent to or stored by <a>Koofr</a>. Only you can decrypt and access your Vault files."
              values={{
                a: (chunks) => (
                  <a href="https://koofr.eu" target="_blank" rel="noopener">
                    {chunks}
                  </a>
                ),
              }}
            />
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
              <FormattedMessage
                id="web.landing_page.section_6.title"
                description="Section title highlighting that Koofr Vault is fully open source."
                defaultMessage="Fully open source"
              />
            </h2>

            <p
              className={css`
                font-size: 18px;
                color: #f4f5f5;
                margin: 0 0 32px;
              `}
            >
              <FormattedMessage
                id="web.landing_page.section_6.content_1"
                description="Section paragraph about open source code and encryption primitives."
                defaultMessage="Koofr Vault is <a>open source</a>, so you can always check that the code does exactly what is promised - and nothing more. File encryption is performed using NaCl SecretBox, which uses XSalsa20 cipher and Poly1305 for ensuring integrity."
                values={{
                  a: (chunks) => (
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
                      {chunks}
                    </a>
                  ),
                }}
              />
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
              <FormattedMessage
                id="web.landing_page.section_6.content_2"
                description="Section paragraph about rclone compatibility."
                defaultMessage="It is compatible with <a>rclone</a> . This means that you can download your encrypted files and decrypt them locally using the rclone command-line tool."
                values={{
                  a: (chunks) => (
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
                      {chunks}
                    </a>
                  ),
                }}
              />
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
            <FormattedMessage
              id="web.landing_page.section_7.title"
              description="Feature highlight label for client-side encryption."
              defaultMessage="Client-side encryption"
            />
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
            <FormattedMessage
              id="web.landing_page.section_8.title"
              description="Feature highlight label for verifiable source code."
              defaultMessage="Verifiable source code"
            />
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
            <FormattedMessage
              id="web.landing_page.section_9.title"
              description="Feature highlight label for zero-knowledge design."
              defaultMessage="Zero knowledge"
            />
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
            <FormattedMessage
              id="web.landing_page.section_10.title"
              description="Call-to-action section title encouraging users to start using Koofr Vault."
              defaultMessage="Unlock your Vault"
            />
          </h2>

          <p
            className={css`
              font-size: 18px;
              color: #011722;
              margin: 0 0 43px;
              text-align: center;
            `}
          >
            <FormattedMessage
              id="web.landing_page.section_10.description"
              description="Call-to-action description mentioning Koofr plans."
              defaultMessage="Start encrypting your cloud storage files in just a few minutes. Included in all <a>Koofr plans</a>."
              values={{
                a: (chunks) => (
                  <a
                    href="https://koofr.eu/pricing"
                    target="_blank"
                    rel="noopener"
                  >
                    {chunks}
                  </a>
                ),
              }}
            />
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
            <FormattedMessage
              id="web.landing_page.get_started.button"
              description="Primary call-to-action button on the landing page that starts login."
              defaultMessage="Get started"
            />
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
          <FormattedMessage
            id="web.landing_page.section_10.questions"
            description="Support link text for questions about Koofr Vault on the landing page."
            defaultMessage="Have questions about Koofr Vault? <a>Find answers here</a>."
            values={{
              a: (chunks) => (
                <a
                  href="https://koofr.eu/help/koofr-vault"
                  target="_blank"
                  rel="noopener"
                >
                  {chunks}
                </a>
              ),
            }}
          />
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
          <FormattedMessage
            id="web.landing_page.copyright.text"
            description="Footer copyright line on the landing page with company link and current year."
            defaultMessage="© {year}. <a>Koofr d.o.o.</a> all rights reserved."
            values={{
              year: new Date().getFullYear(),
              a: (chunks) => (
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
                  {chunks}
                </a>
              ),
            }}
          />
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
LandingPageOfficial.displayName = 'LandingPageOfficial';
