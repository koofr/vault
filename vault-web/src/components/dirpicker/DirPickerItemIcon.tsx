import { css } from '@emotion/css';
import React, { ComponentType, memo } from 'react';

import { ReactComponent as DirPickerItemBookmarkHoverIcon } from '../../assets/images/dir-picker-item-bookmark-hover.svg';
import { ReactComponent as DirPickerItemBookmarkIcon } from '../../assets/images/dir-picker-item-bookmark.svg';
import { ReactComponent as DirPickerItemDesktopHoverIcon } from '../../assets/images/dir-picker-item-desktop-hover.svg';
import { ReactComponent as DirPickerItemDesktopOfflineHoverIcon } from '../../assets/images/dir-picker-item-desktop-offline-hover.svg';
import { ReactComponent as DirPickerItemDesktopOfflineIcon } from '../../assets/images/dir-picker-item-desktop-offline.svg';
import { ReactComponent as DirPickerItemDesktopIcon } from '../../assets/images/dir-picker-item-desktop.svg';
import { ReactComponent as DirPickerItemDropboxHoverIcon } from '../../assets/images/dir-picker-item-dropbox-hover.svg';
import { ReactComponent as DirPickerItemDropboxIcon } from '../../assets/images/dir-picker-item-dropbox.svg';
import { ReactComponent as DirPickerItemExportHoverIcon } from '../../assets/images/dir-picker-item-export-hover.svg';
import { ReactComponent as DirPickerItemExportIcon } from '../../assets/images/dir-picker-item-export.svg';
import { ReactComponent as DirPickerItemFolderHoverIcon } from '../../assets/images/dir-picker-item-folder-hover.svg';
import { ReactComponent as DirPickerItemFolderIcon } from '../../assets/images/dir-picker-item-folder.svg';
import { ReactComponent as DirPickerItemGoogledriveHoverIcon } from '../../assets/images/dir-picker-item-googledrive-hover.svg';
import { ReactComponent as DirPickerItemGoogledriveIcon } from '../../assets/images/dir-picker-item-googledrive.svg';
import { ReactComponent as DirPickerItemHostedHoverIcon } from '../../assets/images/dir-picker-item-hosted-hover.svg';
import { ReactComponent as DirPickerItemHostedIcon } from '../../assets/images/dir-picker-item-hosted.svg';
import { ReactComponent as DirPickerItemImportHoverIcon } from '../../assets/images/dir-picker-item-import-hover.svg';
import { ReactComponent as DirPickerItemImportIcon } from '../../assets/images/dir-picker-item-import.svg';
import { ReactComponent as DirPickerItemOnedriveHoverIcon } from '../../assets/images/dir-picker-item-onedrive-hover.svg';
import { ReactComponent as DirPickerItemOnedriveIcon } from '../../assets/images/dir-picker-item-onedrive.svg';
import { ReactComponent as DirPickerItemRepoHoverIcon } from '../../assets/images/dir-picker-item-repo-hover.svg';
import { ReactComponent as DirPickerItemRepoIcon } from '../../assets/images/dir-picker-item-repo.svg';
import { ReactComponent as DirPickerItemSharedHoverIcon } from '../../assets/images/dir-picker-item-shared-hover.svg';
import { ReactComponent as DirPickerItemSharedIcon } from '../../assets/images/dir-picker-item-shared.svg';
import { DirPickerItemType } from '../../vault-wasm/vault-wasm';

export const itemTypeToIcon: {
  [key in DirPickerItemType]: [
    ComponentType<React.SVGProps<SVGSVGElement>>,
    ComponentType<React.SVGProps<SVGSVGElement>>
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
