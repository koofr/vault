/* eslint-disable react-hooks/rules-of-hooks */
import { useRepoFilesDetailsFileBlobUrl } from './useRepoFilesDetailsFileBlobUrl';
import { useRepoFilesDetailsFileDesktopUrl } from './useRepoFilesDetailsFileDesktopUrl';

export function useRepoFilesDetailsFileUrl(
  detailsId: number,
): string | undefined {
  if (import.meta.env.VITE_VAULT_APP === 'desktop') {
    return useRepoFilesDetailsFileDesktopUrl(detailsId);
  } else {
    return useRepoFilesDetailsFileBlobUrl(detailsId);
  }
}
