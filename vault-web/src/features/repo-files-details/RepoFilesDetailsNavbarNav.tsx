import { memo } from 'react';
import { FormattedMessage } from 'react-intl';
import { Link } from 'react-router-dom';

import FilesEditHoverIcon from '../../assets/images/files-edit-hover.svg?react';
import FilesEditIcon from '../../assets/images/files-edit.svg?react';
import FilesRenameHoverIcon from '../../assets/images/files-rename-hover.svg?react';
import FilesRenameIcon from '../../assets/images/files-rename.svg?react';
import FilesToolbarDeleteHoverIcon from '../../assets/images/files-toolbar-delete-hover.svg?react';
import FilesToolbarDeleteIcon from '../../assets/images/files-toolbar-delete.svg?react';
import FilesToolbarDownloadHoverIcon from '../../assets/images/files-toolbar-download-hover.svg?react';
import FilesToolbarDownloadIcon from '../../assets/images/files-toolbar-download.svg?react';
import {
  NavbarNavToolbar,
  NavbarNavToolbarItem,
} from '../../components/navbar/NavbarNavToolbar';
import { useIsMobile } from '../../components/useIsMobile';
import { RepoFilesDetailsInfo } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { downloadFile } from '../repo-files/repoFilesActions';
import {
  fileCategoryHasDetailsEdit,
  repoFilesDetailsLink,
} from '../repo-files/selectors';

export const RepoFilesDetailsNavbarNav = memo<{
  detailsId: number;
  info: RepoFilesDetailsInfo;
}>(({ detailsId, info }) => {
  const isMobile = useIsMobile();
  const webVault = useWebVault();

  return (
    <NavbarNavToolbar>
      {info.isEditing ? (
        <>
          <NavbarNavToolbarItem
            icon={<FilesRenameIcon role="img" />}
            iconHover={<FilesRenameHoverIcon role="img" />}
            onClick={() => {
              webVault.repoFilesDetailsSave(detailsId);
            }}
            disabled={!info.canSave}
          >
            <FormattedMessage
              id="web.repo_files_details.navbar.save.button"
              description="Navbar button label to save changes while editing a file."
              defaultMessage="Save"
            />
          </NavbarNavToolbarItem>
        </>
      ) : info.fileExists ? (
        <>
          {fileCategoryHasDetailsEdit(info.fileCategory) &&
          info.repoId !== undefined &&
          info.encryptedPath !== undefined ? (
            <NavbarNavToolbarItem
              as={Link}
              to={repoFilesDetailsLink(info.repoId, info.encryptedPath, true)}
              icon={<FilesEditIcon role="img" />}
              iconHover={<FilesEditHoverIcon role="img" />}
            >
              <FormattedMessage
                id="web.repo_files_details.navbar.edit.button"
                description="Navbar button label to enter edit mode for a file."
                defaultMessage="Edit"
              />
            </NavbarNavToolbarItem>
          ) : null}
          <NavbarNavToolbarItem
            icon={<FilesToolbarDownloadIcon role="img" />}
            iconHover={<FilesToolbarDownloadHoverIcon role="img" />}
            onClick={() => {
              if (
                info.repoId !== undefined &&
                info.encryptedPath !== undefined
              ) {
                // eslint-disable-next-line @typescript-eslint/no-floating-promises
                downloadFile(
                  webVault,
                  info.repoId,
                  info.encryptedPath,
                  isMobile,
                );
              }
            }}
          >
            <FormattedMessage
              id="web.repo_files_details.navbar.download.button"
              description="Navbar button label to download the current file."
              defaultMessage="Download"
            />
          </NavbarNavToolbarItem>
          <NavbarNavToolbarItem
            icon={<FilesRenameIcon role="img" />}
            iconHover={<FilesRenameHoverIcon role="img" />}
            onClick={() => {
              if (
                info.repoId !== undefined &&
                info.encryptedPath !== undefined
              ) {
                webVault.repoFilesRenameFile(info.repoId, info.encryptedPath);
              }
            }}
          >
            <FormattedMessage
              id="web.repo_files_details.navbar.rename.button"
              description="Navbar button label to rename the current file."
              defaultMessage="Rename"
            />
          </NavbarNavToolbarItem>
          <NavbarNavToolbarItem
            icon={<FilesToolbarDeleteIcon role="img" />}
            iconHover={<FilesToolbarDeleteHoverIcon role="img" />}
            onClick={() => {
              webVault.repoFilesDetailsDelete(detailsId);
            }}
          >
            <FormattedMessage
              id="web.repo_files_details.navbar.delete.button"
              description="Navbar button label to delete the current file."
              defaultMessage="Delete"
            />
          </NavbarNavToolbarItem>
        </>
      ) : null}
    </NavbarNavToolbar>
  );
});
RepoFilesDetailsNavbarNav.displayName = 'RepoFilesDetailsNavbarNav';
