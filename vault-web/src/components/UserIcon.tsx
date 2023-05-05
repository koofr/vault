import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { memo, useMemo } from 'react';

import { User } from '../vault-wasm/vault-wasm';
import { useSubscribe } from '../webVault/useSubscribe';
import { useWebVault } from '../webVault/useWebVault';

const UserIconLoading = memo(() => {
  const theme = useTheme();

  return (
    <div
      className={css`
        width: 32px;
        height: 32px;
        border-radius: 3px;
        background-color: ${theme.colors.hover};
      `}
    />
  );
});

const UserIconFallback = memo<{ user: User }>(({ user }) => {
  const theme = useTheme();
  const initial =
    user.fullName.length > 0 ? user.fullName[0].toUpperCase() : '';

  return (
    <div
      className={css`
        width: 32px;
        height: 32px;
        border-radius: 3px;
        display: flex;
        justify-content: center;
        align-items: center;
        font-size: 16px;
        font-weight: 600;
        background-color: ${theme.colors.border};
        color: ${theme.colors.textLight};
      `}
    >
      {initial}
    </div>
  );
});

const UserIconUser = memo<{ user: User }>(({ user }) => {
  const webVault = useWebVault();
  useMemo(() => webVault.userEnsureProfilePicture(), [webVault]);
  const [profilePictureLoaded] = useSubscribe(
    (v, cb) => v.userProfilePictureLoadedSubscribe(cb),
    (v) => v.userProfilePictureLoadedData,
    []
  );
  const profilePictureUrl = useMemo(() => {
    if (profilePictureLoaded) {
      const profilePictureArray = webVault.userGetProfilePicture();

      if (profilePictureArray === undefined) {
        return undefined;
      }

      return URL.createObjectURL(new Blob([profilePictureArray]));
    } else {
      return undefined;
    }
  }, [webVault, profilePictureLoaded]);

  if (!profilePictureLoaded) {
    return <UserIconLoading />;
  }

  return profilePictureUrl ? (
    <img
      src={profilePictureUrl}
      alt={user.fullName}
      className={css`
        width: 32px;
        height: 32px;
        border-radius: 3px;
      `}
    />
  ) : (
    <UserIconFallback user={user} />
  );
});

export const UserIcon = memo(() => {
  const [user] = useSubscribe(
    (v, cb) => v.userSubscribe(cb),
    (v) => v.userData,
    []
  );

  return user !== undefined ? (
    <UserIconUser key={user.id} user={user} />
  ) : (
    <UserIconLoading />
  );
});
