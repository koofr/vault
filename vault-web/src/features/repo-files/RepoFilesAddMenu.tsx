import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { useDropdownMenu } from '@restart/ui/DropdownMenu';
import { memo, useCallback } from 'react';

import { Menu, MenuItem } from '../../components/menu/Menu';
import { useMenuUpdate } from '../../components/menu/useMenuUpdate';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { useRepoFilesUploadApi } from './RepoFilesUploadForm';

export const UploadFileItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const uploadApi = useRepoFilesUploadApi();
  const uploadFile = useCallback(() => {
    hide();

    uploadApi.uploadFile?.();
  }, [hide, uploadApi]);

  return <MenuItem onClick={uploadFile}>Upload file</MenuItem>;
});

export const UploadDirItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const uploadApi = useRepoFilesUploadApi();
  const uploadFolder = useCallback(() => {
    hide();

    uploadApi.uploadDir?.();
  }, [hide, uploadApi]);

  return <MenuItem onClick={uploadFolder}>Upload folder</MenuItem>;
});

export const CreateDirItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const createDir = useCallback(
    () => webVault.repoFilesBrowsersCreateDir(browserId),
    [webVault, browserId]
  );

  return (
    <>
      <MenuItem
        onClick={() => {
          hide();
          createDir();
        }}
      >
        Create folder
      </MenuItem>
    </>
  );
});

export const RepoFilesAddMenuContent = memo<{
  hide: () => void;
}>(({ hide }) => {
  return (
    <>
      <UploadFileItem hide={hide} />
      <UploadDirItem hide={hide} />
      <CreateDirItem hide={hide} />
    </>
  );
});

export const RepoFilesAddMenu = memo(() => {
  const theme = useTheme();
  const [props, { show, popper, toggle }] = useDropdownMenu();
  useMenuUpdate(show, popper);

  return (
    <Menu
      isVisible={show}
      {...props}
      className={css`
        width: 230px;
        z-index: ${theme.zindex.repoFilesAddMenu};
      `}
    >
      <RepoFilesAddMenuContent hide={() => toggle?.(false)} />
    </Menu>
  );
});
