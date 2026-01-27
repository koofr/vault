import { css } from '@emotion/css';
import { useTheme } from '@emotion/react';
import { useDropdownMenu } from '@restart/ui/DropdownMenu';
import { format } from 'date-fns/format';
import { memo, useCallback } from 'react';
import { FormattedMessage, useIntl } from 'react-intl';
import { useNavigate } from 'react-router-dom';

import { Menu, MenuDivider, MenuItem } from '../../components/menu/Menu';
import { useMenuUpdate } from '../../components/menu/useMenuUpdate';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { useRepoFilesUploadApi } from './RepoFilesUploadApi';
import { repoFilesDetailsLink } from './selectors';

export const UploadFileItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const uploadApi = useRepoFilesUploadApi();
  const uploadFile = useCallback(() => {
    hide();

    uploadApi.uploadFile?.();
  }, [hide, uploadApi]);

  return (
    <MenuItem onClick={uploadFile}>
      <FormattedMessage
        id="web.repo_files_add_menu.upload_file.menu_item"
        description="Menu item label to upload files from the Add menu in the file browser."
        defaultMessage="Upload file"
      />
    </MenuItem>
  );
});
UploadFileItem.displayName = 'UploadFileItem';

export const UploadDirItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const uploadApi = useRepoFilesUploadApi();
  const uploadFolder = useCallback(() => {
    hide();

    uploadApi.uploadDir?.();
  }, [hide, uploadApi]);

  return (
    <MenuItem onClick={uploadFolder}>
      <FormattedMessage
        id="web.repo_files_add_menu.upload_folder.menu_item"
        description="Menu item label to upload a folder from the Add menu in the file browser."
        defaultMessage="Upload folder"
      />
    </MenuItem>
  );
});
UploadDirItem.displayName = 'UploadDirItem';

export const CreateDirItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const createDir = useCallback(
    () => webVault.repoFilesBrowsersCreateDir(browserId),
    [webVault, browserId],
  );

  return (
    <>
      <MenuItem
        onClick={() => {
          hide();
          createDir();
        }}
      >
        <FormattedMessage
          id="web.repo_files_add_menu.create_dir.menu_item"
          description="Menu item label to create a new folder from the Add menu in the file browser."
          defaultMessage="Create folder"
        />
      </MenuItem>
    </>
  );
});
CreateDirItem.displayName = 'CreateDirItem';

export const CreateTextFileItem = memo<{
  hide: () => void;
}>(({ hide }) => {
  const intl = useIntl();
  const navigate = useNavigate();
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();

  const createFile = useCallback(() => {
    const { repoId } = webVault.repoFilesBrowsersInfo(browserId)!;

    const name =
      intl.formatMessage(
        {
          id: 'web.repo_files.create_text_file.default_filename',
          description:
            'Default base filename used when creating a new text file in the file browser.',
          defaultMessage: 'new text file {date}',
        },
        {
          // Locale is not specified here because only numbers are used in the
          // format.
          date: format(new Date(), 'yyyyMMddHHmmss'),
        },
      ) + '.txt';

    // eslint-disable-next-line @typescript-eslint/no-floating-promises
    webVault.repoFilesBrowsersCreateFile(browserId, name).then(async (path) => {
      if (path !== undefined) {
        await navigate(repoFilesDetailsLink(repoId!, path, true));
      }
    });
  }, [intl, webVault, browserId, navigate]);

  return (
    <>
      <MenuItem
        onClick={() => {
          hide();
          createFile();
        }}
      >
        <FormattedMessage
          id="web.repo_files_add_menu.create_text_file.menu_item"
          description="Menu item label to create a new text file from the Add menu in the file browser."
          defaultMessage="Create new text file"
        />
      </MenuItem>
    </>
  );
});
CreateTextFileItem.displayName = 'CreateTextFileItem';

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
RepoFilesAddMenuContent.displayName = 'RepoFilesAddMenuContent';

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
RepoFilesAddMenu.displayName = 'RepoFilesAddMenu';
