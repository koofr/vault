import { saveAs } from 'file-saver';
import streamSaver from 'streamsaver';

import { FileStream } from '../vault-wasm/vault-wasm';

export function downloadStream(stream: FileStream) {
  if (stream.stream !== undefined) {
    const size =
      stream.size.type === 'Exact'
        ? parseInt(stream.size.size as any, 10)
        : undefined;

    const fileStream = streamSaver.createWriteStream(stream.name, {
      size,
    });

    stream.stream.pipeTo(fileStream).catch(() => {});
  } else if (stream.blob !== undefined) {
    saveAs(stream.blob, stream.name);
  }
}
