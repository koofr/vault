import { memo, useCallback } from 'react';
import { Link } from 'react-router-dom';

import FilesEditHoverIcon from '../../assets/images/files-edit-hover.svg?react';
import FilesEditIcon from '../../assets/images/files-edit.svg?react';
import FilesRenameHoverIcon from '../../assets/images/files-rename-hover.svg?react';
import FilesRenameIcon from '../../assets/images/files-rename.svg?react';
import FilesToolbarCopyHoverIcon from '../../assets/images/files-toolbar-copy-hover.svg?react';
import FilesToolbarCopyIcon from '../../assets/images/files-toolbar-copy.svg?react';
import FilesToolbarDeleteHoverIcon from '../../assets/images/files-toolbar-delete-hover.svg?react';
import FilesToolbarDeleteIcon from '../../assets/images/files-toolbar-delete.svg?react';
import FilesToolbarDownloadHoverIcon from '../../assets/images/files-toolbar-download-hover.svg?react';
import FilesToolbarDownloadIcon from '../../assets/images/files-toolbar-download.svg?react';
import FilesToolbarMoveHoverIcon from '../../assets/images/files-toolbar-move-hover.svg?react';
import FilesToolbarMoveIcon from '../../assets/images/files-toolbar-move.svg?react';
import InfoHoverIcon from '../../assets/images/info-hover.svg?react';
import InfoIcon from '../../assets/images/info.svg?react';
import {
  Toolbar,
  ToolbarCancelItem,
  ToolbarItem,
} from '../../components/toolbar/Toolbar';
import { useIsMobile } from '../../components/useIsMobile';
import { RepoFilesBrowserInfo } from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { downloadSelected } from './repoFilesActions';
import { fileHasDetailsEdit, repoFilesDetailsLink } from './selectors';
import { FormattedMessage } from 'react-intl';

export const RepoFilesToolbar = memo<{
  info: RepoFilesBrowserInfo;
  onInfoClick: () => void;
}>(({ info, onInfoClick }) => {
  const isMobile = useIsMobile();
  const webVault = useWebVault();
  const browserId = useRepoFilesBrowserId();
  const selectedFile =
    info.selectedFile !== undefined ? info.selectedFile : undefined;
  const renameSelectedFile = useCallback(() => {
    if (selectedFile !== undefined) {
      webVault.repoFilesRenameFile(
        selectedFile.repoId,
        selectedFile.encryptedPath,
      );
    }
  }, [webVault, selectedFile]);

  return (
    <Toolbar>
      {info.selectedFile !== undefined ? (
        <ToolbarItem
          icon={<InfoIcon role="img" />}
          iconHover={<InfoHoverIcon role="img" />}
          onClick={onInfoClick}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.info.button"
            description="Toolbar button label in the file list to open the info sidebar for the selected item."
            defaultMessage="Info"
          />
        </ToolbarItem>
      ) : null}
      {isMobile && selectedFile !== undefined ? (
        <ToolbarItem
          icon={<FilesRenameIcon role="img" />}
          iconHover={<FilesRenameHoverIcon role="img" />}
          onClick={renameSelectedFile}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.rename.button"
            description="Toolbar button label on mobile to rename the selected file or folder."
            defaultMessage="Rename"
          />
        </ToolbarItem>
      ) : null}
      {info.canCopySelected ? (
        <ToolbarItem
          icon={<FilesToolbarCopyIcon role="img" />}
          iconHover={<FilesToolbarCopyHoverIcon role="img" />}
          onClick={() => {
            webVault.repoFilesBrowsersMoveSelected(browserId, 'Copy');
          }}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.copy.button"
            description="Toolbar button label to copy the selected files or folders."
            defaultMessage="Copy"
          />
        </ToolbarItem>
      ) : null}
      {info.canMoveSelected ? (
        <ToolbarItem
          icon={<FilesToolbarMoveIcon role="img" />}
          iconHover={<FilesToolbarMoveHoverIcon role="img" />}
          onClick={() => {
            webVault.repoFilesBrowsersMoveSelected(browserId, 'Move');
          }}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.move.button"
            description="Toolbar button label to move the selected files or folders."
            defaultMessage="Move"
          />
        </ToolbarItem>
      ) : null}
      {info.canDownloadSelected ? (
        <ToolbarItem
          icon={<FilesToolbarDownloadIcon role="img" />}
          iconHover={<FilesToolbarDownloadHoverIcon role="img" />}
          onClick={() => {
            // eslint-disable-next-line @typescript-eslint/no-floating-promises
            downloadSelected(webVault, browserId, isMobile);
          }}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.download.button"
            description="Toolbar button label to download the selected files or folders."
            defaultMessage="Download"
          />
        </ToolbarItem>
      ) : null}
      {info.canDeleteSelected ? (
        <ToolbarItem
          icon={<FilesToolbarDeleteIcon role="img" />}
          iconHover={<FilesToolbarDeleteHoverIcon role="img" />}
          onClick={() => {
            webVault.repoFilesBrowsersDeleteSelected(browserId);
          }}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.delete.button"
            description="Toolbar button label to delete the selected files or folders."
            defaultMessage="Delete"
          />
        </ToolbarItem>
      ) : null}
      {info.selectedFile !== undefined &&
      fileHasDetailsEdit(info.selectedFile) ? (
        <ToolbarItem
          as={Link}
          to={repoFilesDetailsLink(
            info.selectedFile.repoId,
            info.selectedFile.encryptedPath,
            true,
          )}
          icon={<FilesEditIcon role="img" />}
          iconHover={<FilesEditHoverIcon role="img" />}
        >
          <FormattedMessage
            id="web.repo_files_toolbar.edit_text.button"
            description="Toolbar button label to open the text editor for the selected text file."
            defaultMessage="Edit text"
          />
        </ToolbarItem>
      ) : null}
      {info.selectedCount > 0 ? (
        <ToolbarCancelItem
          onClick={() => webVault.repoFilesBrowsersClearSelection(browserId)}
        />
      ) : null}
    </Toolbar>
  );
});
RepoFilesToolbar.displayName = 'RepoFilesToolbar';
