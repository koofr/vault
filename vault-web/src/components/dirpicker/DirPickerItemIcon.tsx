import { css } from '@emotion/css';
import React, { ComponentType, memo } from 'react';

import DirPickerItemBookmarkHoverIcon from '../../assets/images/dir-picker-item-bookmark-hover.svg?react';
import DirPickerItemBookmarkIcon from '../../assets/images/dir-picker-item-bookmark.svg?react';
import DirPickerItemDesktopHoverIcon from '../../assets/images/dir-picker-item-desktop-hover.svg?react';
import DirPickerItemDesktopOfflineHoverIcon from '../../assets/images/dir-picker-item-desktop-offline-hover.svg?react';
import DirPickerItemDesktopOfflineIcon from '../../assets/images/dir-picker-item-desktop-offline.svg?react';
import DirPickerItemDesktopIcon from '../../assets/images/dir-picker-item-desktop.svg?react';
import DirPickerItemDropboxHoverIcon from '../../assets/images/dir-picker-item-dropbox-hover.svg?react';
import DirPickerItemDropboxIcon from '../../assets/images/dir-picker-item-dropbox.svg?react';
import DirPickerItemExportHoverIcon from '../../assets/images/dir-picker-item-export-hover.svg?react';
import DirPickerItemExportIcon from '../../assets/images/dir-picker-item-export.svg?react';
import DirPickerItemFolderHoverIcon from '../../assets/images/dir-picker-item-folder-hover.svg?react';
import DirPickerItemFolderIcon from '../../assets/images/dir-picker-item-folder.svg?react';
import DirPickerItemGoogledriveHoverIcon from '../../assets/images/dir-picker-item-googledrive-hover.svg?react';
import DirPickerItemGoogledriveIcon from '../../assets/images/dir-picker-item-googledrive.svg?react';
import DirPickerItemHostedHoverIcon from '../../assets/images/dir-picker-item-hosted-hover.svg?react';
import DirPickerItemHostedIcon from '../../assets/images/dir-picker-item-hosted.svg?react';
import DirPickerItemImportHoverIcon from '../../assets/images/dir-picker-item-import-hover.svg?react';
import DirPickerItemImportIcon from '../../assets/images/dir-picker-item-import.svg?react';
import DirPickerItemOnedriveHoverIcon from '../../assets/images/dir-picker-item-onedrive-hover.svg?react';
import DirPickerItemOnedriveIcon from '../../assets/images/dir-picker-item-onedrive.svg?react';
import DirPickerItemRepoHoverIcon from '../../assets/images/dir-picker-item-repo-hover.svg?react';
import DirPickerItemRepoIcon from '../../assets/images/dir-picker-item-repo.svg?react';
import DirPickerItemSharedHoverIcon from '../../assets/images/dir-picker-item-shared-hover.svg?react';
import DirPickerItemSharedIcon from '../../assets/images/dir-picker-item-shared.svg?react';
import { DirPickerItemType } from '../../vault-wasm/vault-wasm';

export const itemTypeToIcon: {
  [key in DirPickerItemType]: [
    ComponentType<React.SVGProps<SVGSVGElement>>,
    ComponentType<React.SVGProps<SVGSVGElement>>,
  ];
} = {
  Bookmark: [DirPickerItemBookmarkIcon, DirPickerItemBookmarkHoverIcon],
  Bookmarks: [DirPickerItemBookmarkIcon, DirPickerItemBookmarkHoverIcon],
  DesktopOffline: [
    DirPickerItemDesktopOfflineIcon,
    DirPickerItemDesktopOfflineHoverIcon,
  ],
  Desktop: [DirPickerItemDesktopIcon, DirPickerItemDesktopHoverIcon],
  Dropbox: [DirPickerItemDropboxIcon, DirPickerItemDropboxHoverIcon],
  Export: [DirPickerItemExportIcon, DirPickerItemExportHoverIcon],
  Folder: [DirPickerItemFolderIcon, DirPickerItemFolderHoverIcon],
  Googledrive: [
    DirPickerItemGoogledriveIcon,
    DirPickerItemGoogledriveHoverIcon,
  ],
  Hosted: [DirPickerItemHostedIcon, DirPickerItemHostedHoverIcon],
  Import: [DirPickerItemImportIcon, DirPickerItemImportHoverIcon],
  Onedrive: [DirPickerItemOnedriveIcon, DirPickerItemOnedriveHoverIcon],
  Shared: [DirPickerItemSharedIcon, DirPickerItemSharedHoverIcon],
  Repo: [DirPickerItemRepoIcon, DirPickerItemRepoHoverIcon],
};

export const DirPickerItemIcon = memo<{
  itemType: DirPickerItemType;
  hoverSelector: string;
}>(({ itemType, hoverSelector }) => {
  const [Icon, HoverIcon] = itemTypeToIcon[itemType];

  return (
    <>
      <Icon
        className={css`
          display: inline;

          ${hoverSelector} {
            display: none;
          }
        `}
      />
      <HoverIcon
        className={css`
          display: none;

          ${hoverSelector} {
            display: inline;
          }
        `}
      />
    </>
  );
});
