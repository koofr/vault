import { memo, useCallback } from 'react';

import { ErrorComponent } from '../../../components/ErrorComponent';
import { LoadingCircle } from '../../../components/LoadingCircle';
import { TextEditorLazy } from '../../../components/TextEditorLazy';
import { Status } from '../../../vault-wasm/vault-wasm';
import { useWebVault } from '../../../webVault/useWebVault';

import { useRepoFilesDetailsString } from './useRepoFilesDetailsBytes';

export const RepoFilesDetailsTextEditor = memo<{
  detailsId: number;
  fileName: string;
  contentStatus: Status;
  isEditing: boolean;
  width: number;
  height: number;
}>(({ detailsId, fileName, contentStatus, isEditing, width, height }) => {
  const webVault = useWebVault();

  const text = useRepoFilesDetailsString(detailsId);

  const onChange = useCallback(
    (newValue: string) => {
      webVault.repoFilesDetailsSetContent(
        detailsId,
        new TextEncoder().encode(newValue),
      );
    },
    [webVault, detailsId],
  );

  return contentStatus.type === 'Error' && !contentStatus.loaded ? (
    <ErrorComponent
      error={contentStatus.error}
      onRetry={() => {
        webVault.repoFilesDetailsLoadContent(detailsId);
      }}
    />
  ) : (contentStatus.type === 'Loading' && !contentStatus.loaded) ||
    text === undefined ? (
    <LoadingCircle />
  ) : (
    <TextEditorLazy
      fileName={fileName}
      text={text}
      isEditing={isEditing}
      width={width}
      height={height}
      onChange={onChange}
    />
  );
});
