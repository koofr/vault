import { saveAs } from 'file-saver';
import streamSaver from 'streamsaver';

import { FileStream } from '../vault-wasm/vault-wasm';

export function downloadStream(stream: FileStream) {
  if (stream.stream !== undefined) {
    const fileStream = streamSaver.createWriteStream(stream.name, {
      size:
        stream.size !== undefined
          ? parseInt(stream.size as any, 10)
          : undefined,
    });

    stream.stream.pipeTo(fileStream);
  } else if (stream.blob !== undefined) {
    saveAs(stream.blob, stream.name);
  }
}
