import { FormEvent, memo, useCallback, useRef } from 'react';
import { useIntl } from 'react-intl';

import { useUploadFiles } from '../transfers/useUploadFiles';

import { useRepoFilesUploadApi } from './RepoFilesUploadApi';

export const RepoFilesUploadFormWeb = memo(() => {
  const intl = useIntl();
  const uploadFiles = useUploadFiles();
  const uploadFormRef = useRef<HTMLFormElement>(null);
  const uploadApi = useRepoFilesUploadApi();
  const uploadFileInputRef = useCallback(
    (el: HTMLInputElement | null) => {
      if (el === null) {
        // eslint-disable-next-line react-hooks/immutability
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
        // eslint-disable-next-line react-hooks/immutability
        uploadApi.uploadDir = undefined;
      } else {
        uploadApi.uploadDir = () => el.click();
      }
    },
    [uploadApi],
  );
  const onUploadFileChange = useCallback(
    (event: FormEvent<HTMLInputElement>) => {
      if (event.currentTarget.files !== null) {
        const files = Array.from(event.currentTarget.files);
        // eslint-disable-next-line @typescript-eslint/no-floating-promises
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
        aria-label={intl.formatMessage({
          id: 'web.repo_files_upload_form.upload_file.aria_label',
          description:
            'Accessibility label for the hidden file input used to upload files.',
          defaultMessage: 'Upload file',
        })}
      />
      <input
        type="file"
        ref={uploadDirInputRef}
        onChange={onUploadFileChange}
        aria-label={intl.formatMessage({
          id: 'web.repo_files_upload_form.upload_dir.aria_label',
          description:
            'Accessibility label for the hidden directory input used to upload folders.',
          defaultMessage: 'Upload folder',
        })}
        {...{ webkitdirectory: '', mozdirectory: '', directory: '' }}
      />
    </form>
  );
});
RepoFilesUploadFormWeb.displayName = 'RepoFilesUploadFormWeb';
