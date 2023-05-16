import { css, cx } from '@emotion/css';
import { useTheme } from '@emotion/react';
import styled from '@emotion/styled';
import RestartUIModal, { ModalProps, ModalHandle } from '@restart/ui/Modal';
import useClickOutside from '@restart/ui/useClickOutside';
import uniqueId from 'lodash/uniqueId';
import {
  ComponentProps,
  createContext,
  memo,
  PropsWithChildren,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useRef,
} from 'react';

import { ReactComponent as ModalCloseHoverIcon } from '../../assets/images/modal-close-hover.svg';
import { ReactComponent as ModalCloseIcon } from '../../assets/images/modal-close.svg';
import { buttonReset } from '../../styles/mixins/buttons';

import { Button } from '../Button';
import { useIsMobile } from '../useIsMobile';

import { ModalsStoreContext } from './Modals';

export type { ModalProps, ModalHandle } from '@restart/ui/Modal';

export const noopModalClose = () => {};

export const ModalCloseContext = createContext<() => void>(noopModalClose);

export const Modal = memo<ModalProps & React.RefAttributes<ModalHandle>>(
  ({ className, children, ...props }) => {
    const theme = useTheme();
    const isMobile = useIsMobile();

    const id = useMemo(uniqueId, []);
    const isVisible = props.show || false;
    const { topModal, addModal, removeModal } = useContext(ModalsStoreContext);
    const isModalAdded = useRef(false);
    useEffect(() => {
      if (isVisible) {
        addModal(id);
        isModalAdded.current = true;
      } else {
        removeModal(id);
        isModalAdded.current = false;
      }

      return () => {
        removeModal(id);
        isModalAdded.current = false;
      };
    }, [addModal, removeModal, id, isVisible]);
    const isTopModal = topModal === id || (isVisible && !isModalAdded.current);

    return (
      <RestartUIModal
        renderBackdrop={(props) => (
          <div
            {...props}
            className={css`
              display: ${isTopModal ? 'block' : 'none'};
              position: fixed;
              top: 0;
              right: 0;
              bottom: 0;
              left: 0;
              z-index: ${theme.zindex.modalBg};
              background-color: ${theme.colors.backdropModal};
              opacity: ${theme.colors.backdropModalAlpha};
            `}
          />
        )}
        className={cx(
          css`
            position: fixed;
            top: 0;
            right: 0;
            bottom: 0;
            left: 0;
            z-index: ${theme.zindex.modal};
            display: ${isTopModal ? 'flex' : 'none'};
            overflow: hidden;
            outline: 0;
            flex-direction: column;
            justify-content: space-around;
            align-items: center;
          `,
          className
        )}
        {...props}
      >
        <div
          className={cx(
            css`
              position: relative;
              display: flex;
              flex-direction: column;
              overflow: hidden;
            `,
            isMobile
              ? css`
                  margin: 0;
                  width: 100%;
                  height: 100%;
                `
              : css`
                  padding: 15px;
                `
          )}
        >
          <ModalCloseContext.Provider value={props.onHide ?? noopModalClose}>
            <ModalContent isTopModal={isTopModal}>{children}</ModalContent>
          </ModalCloseContext.Provider>
        </div>
      </RestartUIModal>
    );
  }
);

const ModalContent = memo<PropsWithChildren<{ isTopModal: boolean }>>(
  ({ isTopModal, children }) => {
    const theme = useTheme();
    const ref = useRef<HTMLDivElement>(null);
    const close = useContext(ModalCloseContext);
    const onClickOutside = useCallback(() => {
      if (isTopModal) {
        close();
      }
    }, [close, isTopModal]);
    useClickOutside(ref, onClickOutside);

    return (
      <div
        ref={ref}
        className={cx(
          css`
            display: flex;
            flex-direction: column;
            position: relative;
            background-color: #fff;
            background-clip: padding-box;
            outline: 0;
            overflow: hidden;
          `,
          theme.isMobile
            ? css`
                width: 100%;
                height: 100%;
                box-shadow: none;
                border: none;
                border-radius: 0;
              `
            : css`
                width: 560px;
                margin: 15px;
                border: 1px solid rgba(0, 0, 0, 0.2);
                border-radius: 3px;
                box-shadow: 0 5px 15px rgba(0, 0, 0, 0.5);
              `
        )}
      >
        {children}
      </div>
    );
  }
);

export const ModalClose = memo(() => {
  const close = useContext(ModalCloseContext);

  return (
    <button
      type="button"
      className={css`
        ${buttonReset}
        width: 32px;
        height: 32px;
        flex-shrink: 0;
      `}
      onClick={close}
      aria-label="Close"
    >
      <div
        className={css`
          display: flex;
          justify-content: center;
          align-items: center;
        `}
      >
        <ModalCloseIcon
          className={css`
            button:hover > div > & {
              display: none;
            }
          `}
          role="img"
        />
        <ModalCloseHoverIcon
          className={css`
            display: none;

            button:hover > div > & {
              display: inline;
            }
          `}
          role="img"
        />
      </div>
    </button>
  );
});

const StyledModalHeader = styled.div`
  padding: 8px;
  border-bottom: 1px solid ${({ theme }) => theme.colors.borderLight};
  display: flex;
  flex-shrink: 0;
`;

export const ModalHeader = memo<ComponentProps<typeof StyledModalHeader>>(
  ({ children, ...props }) => (
    <StyledModalHeader {...props}>
      {children}
      <ModalClose />
    </StyledModalHeader>
  )
);

export const ModalTitle = styled.h5`
  margin: 0;
  padding: 7px 17px;
  font-size: 14px;
  font-weight: 600;
  color: ${({ theme }) => theme.colors.text};
  text-overflow: ellipsis;
  overflow: hidden;
  flex-grow: 1;
`;

export const ModalBody = styled.div`
  display: flex;
  flex-direction: column;
  position: relative;
  padding: 23px 25px;
  flex-grow: 1;
  overflow: ${({ theme }) => (theme.isMobile ? 'auto' : 'visible')};
`;

export const ModalFooter = styled.div`
  display: flex;
  flex-shrink: 0;
  justify-content: space-between;
  padding: 15px 25px;
  border-top: 1px solid
    ${({ theme }) =>
      theme.isMobile ? theme.colors.borderLight : 'transparent'};
`;

export const ModalFooterExtra = styled.div`
  display: flex;
  align-items: center;
  margin-right: auto;
  width: 33%;
  flex-grow: 1;
`;

export const ModalFooterMiddle = styled.div`
  display: flex;
  width: 33%;
`;

export const ModalFooterButtons = styled.div`
  display: flex;
  margin-left: auto;
  width: 33%;
  flex-grow: 1;
  justify-content: flex-end;
`;

export const ModalFooterButton = styled(Button)`
  margin-left: 15px;
`;
