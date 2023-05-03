import { DocumentSizeInfo } from '../../../components/DocumentSize';
import { getNavbarHeight } from '../../../components/navbar/Navbar';
import { isDocumentSizeMobile } from '../../../components/useIsMobile';
import { FileCategory, Status } from '../../../vault-wasm/vault-wasm';

import {
  fileHasImageViewer,
  fileHasPdfViewer,
  fileHasTextEditor,
} from '../selectors';

import { RepoFilesDetailsImageViewer } from './RepoFilesDetailsImageViewer';
import { RepoFilesDetailsPdfViewer } from './RepoFilesDetailsPdfViewer';
import { RepoFilesDetailsTextEditor } from './RepoFilesDetailsTextEditor';

export const getContentEl = (
  detailsId: number,
  fileName: string | undefined,
  fileExt: string | undefined,
  fileCategory: FileCategory | undefined,
  contentStatus: Status | undefined,
  isEditing: boolean,
  documentSize: DocumentSizeInfo
): React.ReactElement | undefined => {
  const isMobile = isDocumentSizeMobile(documentSize);
  const width = documentSize.width;
  const height = documentSize.height - getNavbarHeight(isMobile);

  if (fileHasPdfViewer(fileExt)) {
    return (
      <RepoFilesDetailsPdfViewer
        detailsId={detailsId}
        width={width}
        height={height}
      />
    );
  } else if (fileHasTextEditor(fileCategory) && fileName !== undefined) {
    return (
      <RepoFilesDetailsTextEditor
        detailsId={detailsId}
        fileName={fileName}
        contentStatus={contentStatus}
        isEditing={isEditing}
        width={width}
        height={height}
      />
    );
  } else if (fileHasImageViewer(fileExt) && fileName !== undefined) {
    return (
      <RepoFilesDetailsImageViewer
        detailsId={detailsId}
        fileName={fileName}
        contentStatus={contentStatus}
        width={width}
        height={height}
      />
    );
  }

  return undefined;
};
