import { memo, useCallback } from 'react';
import { Link } from 'react-router-dom';

import { ReactComponent as FilesEditHoverIcon } from '../../assets/images/files-edit-hover.svg';
import { ReactComponent as FilesEditIcon } from '../../assets/images/files-edit.svg';
import { ReactComponent as FilesRenameHoverIcon } from '../../assets/images/files-rename-hover.svg';
import { ReactComponent as FilesRenameIcon } from '../../assets/images/files-rename.svg';
import { ReactComponent as FilesToolbarCopyHoverIcon } from '../../assets/images/files-toolbar-copy-hover.svg';
import { ReactComponent as FilesToolbarCopyIcon } from '../../assets/images/files-toolbar-copy.svg';
import { ReactComponent as FilesToolbarDeleteHoverIcon } from '../../assets/images/files-toolbar-delete-hover.svg';
import { ReactComponent as FilesToolbarDeleteIcon } from '../../assets/images/files-toolbar-delete.svg';
import { ReactComponent as FilesToolbarDownloadHoverIcon } from '../../assets/images/files-toolbar-download-hover.svg';
import { ReactComponent as FilesToolbarDownloadIcon } from '../../assets/images/files-toolbar-download.svg';
import { ReactComponent as FilesToolbarMoveHoverIcon } from '../../assets/images/files-toolbar-move-hover.svg';
import { ReactComponent as FilesToolbarMoveIcon } from '../../assets/images/files-toolbar-move.svg';
import {
  Toolbar,
  ToolbarCancelItem,
  ToolbarItem,
} from '../../components/toolbar/Toolbar';
import { useIsMobile } from '../../components/useIsMobile';
import {
  RepoFilesBrowserInfo,
  RepoFilesMoveMode,
} from '../../vault-wasm/vault-wasm';
import { useWebVault } from '../../webVault/useWebVault';

import { useRepoFilesBrowserId } from './RepoFilesBrowserId';
import { downloadSelected } from './repoFilesActions';
import { fileHasDetailsEdit, repoFilesDetailsLink } from './selectors';

export const RepoFilesToolbar = memo<{ info: RepoFilesBrowserInfo }>(
  ({ info }) => {
    const isMobile = useIsMobile();
    const webVault = useWebVault();
    const browserId = useRepoFilesBrowserId();
    const selectedFile =
      info.selectedFile !== undefined && info.path !== undefined
        ? info.selectedFile
        : undefined;
    const renameSelectedFile = useCallback(() => {
      if (selectedFile !== undefined && selectedFile.path !== undefined) {
        webVault.repoFilesRenameFile(selectedFile.repoId, selectedFile.path);
      }
    }, [webVault, selectedFile]);

    return (
      <Toolbar>
        {isMobile && selectedFile !== undefined ? (
          <ToolbarItem
            icon={<FilesRenameIcon role="img" />}
            iconHover={<FilesRenameHoverIcon role="img" />}
            onClick={renameSelectedFile}
          >
            Rename
          </ToolbarItem>
        ) : null}
        {info.canCopySelected ? (
          <ToolbarItem
            icon={<FilesToolbarCopyIcon role="img" />}
            iconHover={<FilesToolbarCopyHoverIcon role="img" />}
            onClick={() => {
              webVault.repoFilesBrowsersMoveSelected(
                browserId,
                RepoFilesMoveMode.Copy
              );
            }}
          >
            Copy
          </ToolbarItem>
        ) : null}
        {info.canMoveSelected ? (
          <ToolbarItem
            icon={<FilesToolbarMoveIcon role="img" />}
            iconHover={<FilesToolbarMoveHoverIcon role="img" />}
            onClick={() => {
              webVault.repoFilesBrowsersMoveSelected(
                browserId,
                RepoFilesMoveMode.Move
              );
            }}
          >
            Move
          </ToolbarItem>
        ) : null}
        {info.canDownloadSelected ? (
          <ToolbarItem
            icon={<FilesToolbarDownloadIcon role="img" />}
            iconHover={<FilesToolbarDownloadHoverIcon role="img" />}
            onClick={() => {
              downloadSelected(webVault, browserId, isMobile);
            }}
          >
            Download
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
            Delete
          </ToolbarItem>
        ) : null}
        {info.selectedFile !== undefined &&
        fileHasDetailsEdit(info.selectedFile) ? (
          <ToolbarItem
            as={Link}
            to={repoFilesDetailsLink(
              info.selectedFile.repoId,
              info.selectedFile.path!,
              true
            )}
            icon={<FilesEditIcon role="img" />}
            iconHover={<FilesEditHoverIcon role="img" />}
          >
            Edit text
          </ToolbarItem>
        ) : null}
        {info.selectedCount > 0 ? (
          <ToolbarCancelItem
            onClick={() => webVault.repoFilesBrowsersClearSelection(browserId)}
          />
        ) : null}
      </Toolbar>
    );
  }
);
