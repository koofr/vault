import { saveAs } from 'file-saver';
import streamSaver from 'streamsaver';

import { FileStream } from '../vault-wasm/vault-wasm';

export function downloadStream(stream: FileStream, name: string) {
  if (stream.stream !== undefined) {
    const fileStream = streamSaver.createWriteStream(name, {
      size: parseInt(stream.size as any, 10),
    });

    stream.stream.pipeTo(fileStream);
  } else if (stream.blob !== undefined) {
    saveAs(stream.blob, name);
  }
}
