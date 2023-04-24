import { css, keyframes } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

const loadingCircle = keyframes`
  0%,
  100% {
    -webkit-transform: scale(0.1);
    transform: scale(0.1);
  }
  50% {
    -webkit-transform: scale(1);
    transform: scale(1);
  }
`;

export const LoadingCircle = memo(() => {
  const theme = useTheme();

  return (
    <div
      className={css`
        display: flex;
        justify-content: center;
        align-items: center;
        flex-grow: 1;
        margin-bottom: 70px + 35px; // navbar height + half circle height
      `}
      aria-label="Loading..."
    >
      <div
        className={css`
          width: 70px;
          height: 70px;
          background-color: ${theme.colors.primary};
          border-radius: 100%;
          animation: ${loadingCircle} 2s infinite ease-out both;

          @media (max-width: 768px) {
            width: 60px;
            height: 60px;
          }
        `}
      ></div>
    </div>
  );
});
