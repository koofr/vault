import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { useDropdownMenu } from '@restart/ui/DropdownMenu';
import format from 'date-fns/format';
import { memo, useCallback } from 'react';
import { useNavigate } from 'react-router-dom';

import { Menu, MenuDivider, MenuItem } from '../../components/menu/Menu';
import { useMenuUpdate } from '../../components/menu/useMenuUpdate';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { useRepoFilesUploadApi } from './RepoFilesUploadForm';
import { repoFilesDetailsLink } from './selectors';

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

export const CreateTextFileItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const navigate = useNavigate();
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const createFile = useCallback(() => {
    const { repoId } = webVault.repoFilesBrowsersInfo(browserId)!;

    const name = `new text file ${format(new Date(), 'yyyyMMddHHmmss')}.txt`;

    webVault.repoFilesBrowsersCreateFile(browserId, name).then((path) => {
      if (path !== undefined) {
        navigate(repoFilesDetailsLink(repoId!, path, true));
      }
    });
  }, [webVault, browserId, navigate]);

  return (
    <>
      <MenuItem
        onClick={() => {
          hide();
          createFile();
        }}
      >
        Create new text file
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
      <MenuDivider />
      <CreateTextFileItem hide={hide} />
    </>
  );
});

export const RepoFilesAddMenu = memo(() => {
  const theme = useTheme();
  const [props, { show, popper, toggle }] = useDropdownMenu({
    popperConfig: {
      strategy: 'fixed',
    },
  });
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
