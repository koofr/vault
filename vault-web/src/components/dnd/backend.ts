import { DragDropManager } from 'dnd-core';
import type {
  HTML5BackendContext,
  HTML5BackendOptions,
} from 'react-dnd-html5-backend';
import { HTML5BackendImpl } from 'react-dnd-html5-backend/dist/HTML5BackendImpl';

export function FolderAwareHTML5Backend(
  manager: DragDropManager,
  context?: HTML5BackendContext,
  options?: HTML5BackendOptions
) {
  const backend = new HTML5BackendImpl(manager, context, options);

  const handleTopDropCaptureOriginal = backend.handleTopDropCapture;

  backend.handleTopDropCapture = function (event: DragEvent) {
    const currentNativeSource: any | null = (backend as any)
      .currentNativeSource;

    if (currentNativeSource != null && currentNativeSource.item != null) {
      try {
        currentNativeSource.item.items =
          event.dataTransfer != null && event.dataTransfer.items != null
            ? Array.from(event.dataTransfer.items)
            : null;
      } catch (e) {
        //
      }
    }

    try {
      return handleTopDropCaptureOriginal.call(backend, event);
    } catch {
      // in MS Edge accessing event.dataTransfer.files or event.dataTransfer.items
      // can throw an exception Access is denied (e.g. user actually cannot
      // read the file that she dropped).
      try {
        (backend as any).enterLeaveCounter.reset();
      } catch {
        //
      }
    }
  };

  return backend;
}
