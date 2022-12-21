import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useCallback } from 'react';

import { buttonReset } from '../../styles/mixins/buttons';
import { useSubscribe } from '../../webVault/useSubscribe';
import { useWebVault } from '../../webVault/useWebVault';

import { Notification } from './Notification';

export const Notifications = memo<{}>(() => {
  const theme = useTheme();
  const webVault = useWebVault();
  const notifications = useSubscribe(
    (v, cb) => v.notificationsSubscribe(cb),
    (v) => v.notificationsData,
    []
  );
  const hasRemoveAll = notifications.length > 1;
  const remove = useCallback(
    (id: number) => {
      webVault.notificationsRemove(id);
    },
    [webVault]
  );
  const removeAll = useCallback(() => {
    webVault.notificationsRemoveAll();
  }, [webVault]);

  if (notifications.length === 0) {
    return null;
  }

  return (
    <div
      className={css`
        z-index: ${theme.zindex.notifications};
        position: fixed;
        right: 0px;
        top: 0px;
        display: flex;
        flex-direction: column;
        padding: 5px;
      `}
    >
      {hasRemoveAll ? (
        <button
          type="button"
          className={css`
            ${buttonReset}
            background-color: #000;
            opacity: 0.85;
            width: 235px;
            padding: 4px 10px;
            margin: 5px;
            display: block;
            border-radius: 5px;
            color: #fff;
            font-size: 11px;
            font-weight: bold;
            text-align: center;
          `}
          onClick={removeAll}
        >
          [ close all ]
        </button>
      ) : undefined}

      {notifications.map((notification) => (
        <Notification
          key={notification.id}
          message={notification.message}
          remove={() => remove(notification.id)}
        />
      ))}
    </div>
  );
});
