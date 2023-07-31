import { css } from '@emotion/css';
import { memo, useEffect } from 'react';

import { buttonReset } from '../../styles/mixins/buttons';

export const Notification = memo<{
  message: string;
  remove: () => void;
  removeAfter: (durationMs: number) => void;
}>(({ message, remove, removeAfter }) => {
  useEffect(() => {
    removeAfter(3000);
  }, [removeAfter]);

  return (
    <div
      className={css`
        background-color: #000;
        opacity: 0.85;
        width: 235px;
        padding: 10px;
        margin: 5px;
        border-radius: 5px;
        min-height: 40px;
      `}
    >
      <button
        type="button"
        className={css`
          ${buttonReset}
          float: right;
          color: #fff;
          font-size: 15px;
          font-weight: bold;
          line-height: 13px;
          padding: 5px;
          margin-top: -5px;
          margin-right: -5px;
        `}
        onClick={remove}
      >
        ×
      </button>
      <div
        className={css`
          color: #fff;
          font-size: 12px;
        `}
      >
        {message}
      </div>
    </div>
  );
});
