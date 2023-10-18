import { css, cx } from '@emotion/css';
import styled from '@emotion/styled';
import useEmblaCarousel from 'embla-carousel-react';
import range from 'lodash/range';
import {
  memo,
  PropsWithChildren,
  useCallback,
  useEffect,
  useState,
} from 'react';

import intro01Image from '../../assets/images/intro/intro-01.png';
import intro012xImage from '../../assets/images/intro/intro-01@2x.png';
import intro02Image from '../../assets/images/intro/intro-02.png';
import googlePlayImage from '../../assets/images/apps/google-play.png';
import googlePlay2xImage from '../../assets/images/apps/google-play@2x.png';
import appStoreImage from '../../assets/images/apps/app-store.png';
import appStore2xImage from '../../assets/images/apps/app-store@2x.png';
import intro022xImage from '../../assets/images/intro/intro-02@2x.png';
import intro03Image from '../../assets/images/intro/intro-03.png';
import intro032xImage from '../../assets/images/intro/intro-03@2x.png';
import intro04Image from '../../assets/images/intro/intro-04.png';
import intro042xImage from '../../assets/images/intro/intro-04@2x.png';
import intro05Image from '../../assets/images/intro/intro-05.png';
import intro052xImage from '../../assets/images/intro/intro-05@2x.png';
import intro06Image from '../../assets/images/intro/intro-06.png';
import intro062xImage from '../../assets/images/intro/intro-06@2x.png';
import intro07Image from '../../assets/images/intro/intro-07.png';
import intro072xImage from '../../assets/images/intro/intro-07@2x.png';
import intro08Image from '../../assets/images/intro/intro-08.png';
import intro082xImage from '../../assets/images/intro/intro-08@2x.png';
import intro09Image from '../../assets/images/intro/intro-09.png';
import intro092xImage from '../../assets/images/intro/intro-09@2x.png';
import introEndImage from '../../assets/images/intro/intro-end.png';
import introEnd2xImage from '../../assets/images/intro/intro-end@2x.png';
import introStartImage from '../../assets/images/intro/intro-start.png';
import introStart2xImage from '../../assets/images/intro/intro-start@2x.png';
import { buttonStyle } from '../../styles/mixins/buttons';
import { imgRetinaBase } from '../../styles/mixins/images';

import { BaseButton } from '../Button';
import { RetinaImage } from '../RetinaImage';
import {
  Modal,
  ModalBody,
  ModalFooter,
  ModalFooterButtons,
  ModalFooterExtra,
  ModalFooterMiddle,
  ModalHeader,
  ModalTitle,
} from '../modal/Modal';
import { useConfig } from '../../config';

const introButtonColor = '#2286f7';
const introButtonColorHover = '#0870e6';

const introButtonStyle = buttonStyle(
  introButtonColor,
  introButtonColor,
  '#ffffff',
  introButtonColorHover,
  introButtonColorHover,
  '#ffffff',
);

const introButtonInlineStyle = cx(
  buttonStyle(
    'transparent',
    'transparent',
    introButtonColor,
    'transparent',
    'transparent',
    introButtonColorHover,
  ),
  css`
    padding-left: 0;
    padding-right: 0;
  `,
);

const bp = {
  small: `@media (max-width: 558px)`,
  normal: `@media (min-width: 559px)`,
};

const IntroStep1 = memo(() => {
  return (
    <div
      className={css`
        padding: 0;
        ${imgRetinaBase(introStartImage, introStart2xImage, 560, 202)}
        background-position: 0 51px;

        ${bp.small} {
          background-size: 100% auto;
        }

        @media (max-width: 558px) {
          width: 100%;
        }

        @media (min-width: 558px) {
          width: 558px;
          height: 300px;
        }
      `}
    >
      <p
        className={css`
          margin: 0;
          font-weight: normal;
          color: #011722;

          ${bp.small} {
            width: 220px;
            padding: 150px 0 0 34px;
            font-size: 16px;
          }

          ${bp.normal} {
            width: 245px;
            padding: 140px 0 0 34px;
            font-size: 18px;
          }
        `}
      >
        Welcome to Koofr Vault: your favourite cloud storage with client-side
        encryption!
      </p>
    </div>
  );
});

const IntroFeatures = styled.div`
  display: flex;
  padding: 25px 0 0;

  ${bp.small} {
    flex-direction: column;
  }

  ${bp.normal} {
    flex-direction: row;
    justify-content: space-between;
    height: 300px;
  }
`;

const IntroFeature = memo<
  PropsWithChildren<{ image: string; image2x: string }>
>(({ image, image2x, children }) => {
  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
        align-items: center;

        ${bp.small} {
          padding: 0 25px 25px;
        }

        ${bp.normal} {
          margin: 0 15px;
          width: 150px;
        }
      `}
    >
      <RetinaImage
        image={image}
        image2x={image2x}
        width={130}
        height={130}
        className={css`
          margin-bottom: 15px;
        `}
      />
      <p
        className={css`
          font-size: 14px;
          color: #011722;
          margin: 0;
          text-align: center;

          & strong {
            font-weight: 600;
          }
        `}
      >
        {children}
      </p>
    </div>
  );
});

const IntroStep2 = memo(() => {
  return (
    <IntroFeatures>
      <IntroFeature image={intro01Image} image2x={intro012xImage}>
        The Vault is like having a box with a unique lock in your trunk: an
        extra layer of security to protect your most sensitive files.
      </IntroFeature>
      <IntroFeature image={intro02Image} image2x={intro022xImage}>
        Vault is{' '}
        <a
          href="https://github.com/koofr/vault"
          target="_blank"
          rel="noreferrer"
        >
          open source
        </a>
        , so you can always check that the code does exactly what is promised -
        and nothing more.
      </IntroFeature>
      <IntroFeature image={intro03Image} image2x={intro032xImage}>
        <a href="https://rclone.org" target="_blank" rel="noreferrer">
          rclone
        </a>{' '}
        compatibility: Download your encrypted files and decrypt them locally
        using the rclone command-line tool.
      </IntroFeature>
    </IntroFeatures>
  );
});

const IntroStep3 = memo(() => {
  return (
    <IntroFeatures>
      <IntroFeature image={intro04Image} image2x={intro042xImage}>
        You can have one or more Safe Boxes in your Vault, with separate Safe
        Keys for each Safe Box.
      </IntroFeature>
      <IntroFeature image={intro05Image} image2x={intro052xImage}>
        Vault encrypts both file names and content, so they are only readable
        inside the Safe Box.
      </IntroFeature>
      <IntroFeature image={intro06Image} image2x={intro062xImage}>
        In your main Koofr app, the files will appear with encrypted file names
        and cannot be opened.
      </IntroFeature>
    </IntroFeatures>
  );
});

const IntroStep4 = memo(() => {
  return (
    <IntroFeatures>
      <IntroFeature image={intro07Image} image2x={intro072xImage}>
        When you create a Safe Box, you'll choose a Safe Key for it.
      </IntroFeature>
      <IntroFeature image={intro08Image} image2x={intro082xImage}>
        Since the Safe Key is used to encrypt the files you store in the Safe
        Box, it cannot be changed later.
      </IntroFeature>
      <IntroFeature image={intro09Image} image2x={intro092xImage}>
        <strong>
          There is no way recover your files if you forget your Safe Key.
        </strong>{' '}
        It is never sent to or stored on Koofr servers.
      </IntroFeature>
    </IntroFeatures>
  );
});

const IntroStep5 = memo(() => {
  const config = useConfig();

  return (
    <div
      className={css`
        display: flex;
        flex-direction: column;
        padding: 0;
        ${imgRetinaBase(introEndImage, introEnd2xImage, 252, 203)}

        ${bp.small} {
          background-position: bottom 15px center;
          width: 100%;
          height: 335px;
        }

        ${bp.normal} {
          background-position: 281px 68px;
          width: 558px;
          height: 300px;
        }
      `}
    >
      <div
        className={css`
          ${bp.normal} {
            display: flex;
            flex-direction: row;
            align-items: center;
            flex-grow: 1;
          }
        `}
      >
        <p
          className={css`
            font-weight: normal;
            color: #011722;

            ${bp.small} {
              width: 245px;
              margin: 0 auto;
              padding: 10px 0 0 0;
              text-align: center;
              font-size: 16px;
            }

            ${bp.normal} {
              width: 200px;
              margin-left: 34px;
              font-size: 18px;
            }
          `}
        >
          Get started with Vault by creating your first Safe Box.
        </p>
      </div>
      {config.appStoreUrl !== undefined ||
      config.googlePlayUrl !== undefined ? (
        <div
          className={css`
            ${bp.small} {
              display: flex;
              flex-direction: row;
              justify-content: center;
              margin-top: 20px;
            }

            ${bp.normal} {
              display: flex;
              flex-direction: row;
              flex-shrink: 0;
              margin-left: 34px;
            }
          `}
        >
          {config.appStoreUrl !== undefined ? (
            <a
              href={config.appStoreUrl}
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

          {config.googlePlayUrl !== undefined ? (
            <a href={config.googlePlayUrl} target="_blank" rel="noreferrer">
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
  );
});

const IntroStep = styled.div`
  display: flex;
  flex-direction: column;
  justify-content: center;
  align-items: center;
  min-height: 100%;
`;

const IntroModalContent = memo<{ hide: () => void }>(({ hide }) => {
  const stepTitles = [
    'Welcome',
    'About Vault',
    'Safe Boxes',
    'Safe Key',
    'Start using Vault',
  ];
  const steps = [
    <IntroStep1 />,
    <IntroStep2 />,
    <IntroStep3 />,
    <IntroStep4 />,
    <IntroStep5 />,
  ];
  const stepsCount = steps.length;
  const [step, setStep] = useState(0);
  const hasPrev = step > 0;
  const isLast = step === stepsCount - 1;
  const [emblaRef, emblaApi] = useEmblaCarousel({
    loop: false,
    speed: 15,
  });
  useEffect(() => {
    if (emblaApi !== undefined) {
      const onSelect = () => {
        const indexNew = emblaApi.selectedScrollSnap();

        if (indexNew === stepsCount) {
          hide();
        } else {
          setStep(indexNew);
        }
      };

      emblaApi.on('select', onSelect);

      return () => {
        emblaApi.off('select', onSelect);
      };
    }
  }, [emblaApi, hide, stepsCount]);
  const prev = useCallback(() => {
    setStep((step) => {
      const newStep = step - 1;

      if (emblaApi !== undefined) {
        emblaApi.scrollTo(newStep);
      }

      return newStep;
    });
  }, [emblaApi]);
  const next = useCallback(() => {
    if (isLast) {
      hide();
    } else {
      setStep((step) => {
        const newStep = step + 1;

        if (emblaApi !== undefined) {
          emblaApi.scrollTo(newStep);
        }

        return newStep;
      });
    }
  }, [hide, isLast, emblaApi]);

  return (
    <>
      <ModalHeader>
        <ModalTitle>{stepTitles[step]}</ModalTitle>
      </ModalHeader>
      <ModalBody
        className={css`
          padding: 0;

          @media (max-width: 768px) {
            height: 100%;
          }
        `}
      >
        <div
          className={css`
            overflow: hidden;
            height: 100%;
          `}
          ref={emblaRef}
        >
          <div
            className={css`
              display: flex;
              align-items: flex-start;
              height: 100%;
            `}
          >
            {steps.map((step, idx) => (
              <div
                key={idx}
                className={css`
                  flex: 0 0 100%;
                  min-width: 0;
                  height: 100%;
                  overflow-x: hidden;
                  overflow-y: auto;
                `}
              >
                <IntroStep>{step}</IntroStep>
              </div>
            ))}
            <div
              className={css`
                flex: 0 0 100%;
                min-width: 0;
                height: 100%;
              `}
            />
          </div>
        </div>
      </ModalBody>
      <ModalFooter>
        <ModalFooterExtra>
          {hasPrev ? (
            <BaseButton
              type="button"
              className={introButtonInlineStyle}
              onClick={prev}
            >
              Back
            </BaseButton>
          ) : null}
        </ModalFooterExtra>
        <ModalFooterMiddle>
          <div
            className={css`
              display: flex;
              margin: auto;
            `}
          >
            {range(0, stepsCount).map((i) => (
              <div
                key={i}
                className={cx(
                  css`
                    background-color: #d4d6d7;
                    display: inline-block;
                    width: 6px;
                    height: 6px;
                    border-radius: 3px;
                    margin: 0 2.5px;
                  `,
                  i === step &&
                    css`
                      background-color: #676f73;
                    `,
                )}
              ></div>
            ))}
          </div>
        </ModalFooterMiddle>
        <ModalFooterButtons>
          <BaseButton type="button" className={introButtonStyle} onClick={next}>
            {isLast ? 'Done' : 'Next'}
          </BaseButton>
        </ModalFooterButtons>
      </ModalFooter>
    </>
  );
});

export interface IntroModalProps {
  isVisible: boolean;
  hide: () => void;
}

export const IntroModal = memo<IntroModalProps>(({ isVisible, hide }) => {
  return (
    <Modal show={isVisible} onHide={hide}>
      {isVisible ? <IntroModalContent hide={hide} /> : <></>}
    </Modal>
  );
});
