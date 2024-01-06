import { FormEvent, memo, useCallback, useRef } from 'react';

import { useUploadFiles } from '../transfers/useUploadFiles';

import { useRepoFilesUploadApi } from './RepoFilesUploadApi';

export const RepoFilesUploadFormWeb = memo(() => {
  const uploadFiles = useUploadFiles();
  const uploadFormRef = useRef<HTMLFormElement>(null);
  const uploadApi = useRepoFilesUploadApi();
  const uploadFileInputRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (el === null) {
        uploadApi.uploadFile = undefined;
      } else {
        uploadApi.uploadFile = () => el.click();
      }
    },
    [uploadApi],
  );
  const uploadDirInputRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (el === null) {
        uploadApi.uploadDir = undefined;
      } else {
        uploadApi.uploadDir = () => el.click();
      }
    },
    [uploadApi],
  );
  const onUploadFileChange = useCallback(
    async (event: FormEvent<HTMLInputElement>) => {
      if (event.currentTarget.files !== null) {
        const files = Array.from(event.currentTarget.files);
        Promise.all(uploadFiles(files));
      }
      if (uploadFormRef.current !== null) {
        // reset the form so that the same file can be uploaded again
        uploadFormRef.current.reset();
      }
    },
    [uploadFiles],
  );

  return (
    <form ref={uploadFormRef} style={{ display: 'none' }}>
      <input
        type="file"
        ref={uploadFileInputRef}
        onChange={onUploadFileChange}
        aria-label="Upload file"
      />
      <input
        type="file"
        ref={uploadDirInputRef}
        onChange={onUploadFileChange}
        aria-label="Upload folder"
        {...{ webkitdirectory: '', mozdirectory: '', directory: '' }}
      />
    </form>
  );
});
