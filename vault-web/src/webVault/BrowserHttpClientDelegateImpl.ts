import { BrowserHttpClientDelegate } from '../vault-wasm/vault-wasm';

export class BrowserHttpClientDelegateImpl
  implements BrowserHttpClientDelegate
{
  async fetch(request: Request): Promise<Response> {
    return await fetch(request);
  }

  xhr(
    request: Request,
    blob: Blob,
    onRequestProgress: (n: number) => void
  ): Promise<Response> {
    return new Promise((resolve, reject) => {
      const xhr = new XMLHttpRequest();

      xhr.open('POST', request.url, true);

      xhr.responseType = 'blob';

      request.headers.forEach((value, key) => {
        xhr.setRequestHeader(key, value);
      });

      let lastLoaded = 0;

      xhr.upload.onprogress = (event: ProgressEvent) => {
        if (event.lengthComputable) {
          const loaded = event.loaded;
          const n = loaded - lastLoaded;

          onRequestProgress(n);

          lastLoaded = loaded;
        }
      };

      xhr.onload = () => {
        const headers: Record<string, string> = {};

        for (const name of [
          'content-type',
          'content-length',
          'X-User-Id',
          'X-Request-Id',
          'X-File-Info',
        ]) {
          const value = xhr.getResponseHeader(name);

          if (value !== null) {
            headers[name] = value;
          }
        }

        resolve(
          new Response(xhr.response, {
            status: xhr.status,
            headers: headers,
          })
        );
      };

      xhr.onerror = () => {
        reject(new Error('XHR error'));
      };

      request.signal.onabort = () => {
        xhr.abort();
      };

      xhr.onabort = () => {};

      xhr.send(blob);
    });
  }
}
