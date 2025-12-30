import { encode, decode } from 'base64-arraybuffer';

import { Callbacks } from './Callbacks';
import { RequestEncryption } from './RequestEncryption';

export class WebVaultClient {
  baseUrl: string;
  requestEncryption: RequestEncryption;
  callbacks: Callbacks;

  callbacksEventSource!: EventSource;

  constructor(
    baseUrl: string,
    requestEncryption: RequestEncryption,
    callbacks: Callbacks,
  ) {
    this.baseUrl = baseUrl;
    this.requestEncryption = requestEncryption;
    this.callbacks = callbacks;
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  getUrl(name: string, params: { [key: string]: any }): string {
    const encryptedRequest = this.requestEncryption.encryptRequest({
      method: 'GET',
      uri: `/WebVault/${name}?${new URLSearchParams(params).toString()}`,
    });

    return `${this.baseUrl}/?req=${encodeURIComponent(encryptedRequest)}`;
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  call(name: string, rawArgs: any[], asyncCall: boolean, bytes: boolean) {
    const args = this.prepareArgs(rawArgs);

    const request = new XMLHttpRequest();
    request.open('POST', `${this.baseUrl}/`, asyncCall);

    const requestBody = this.requestEncryption.encryptRequest({
      method: 'POST',
      uri: `/WebVault/${name}`,
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(args),
    });

    if (asyncCall) {
      return new Promise((resolve, reject) => {
        request.onload = () => {
          resolve(this.handleResponse(request, bytes));
        };

        request.onerror = () => {
          // eslint-disable-next-line @typescript-eslint/prefer-promise-reject-errors
          reject(request);
        };

        request.send(requestBody);
      });
    } else {
      request.send(requestBody);

      // eslint-disable-next-line @typescript-eslint/no-unsafe-return
      return this.handleResponse(request, bytes);
    }
  }

  handleResponse(request: XMLHttpRequest, bytes: boolean) {
    if (request.status === 204) {
      return undefined;
    }

    const { body } = this.requestEncryption.decryptResponse(
      request.responseText,
    );

    if (request.status !== 200 && request.status !== 204) {
      throw new Error(`Call error: ${request.status}: ${body}`);
    }

    if (bytes) {
      return new Uint8Array(decode(body));
    }

    if (body === '') {
      return undefined;
    }

    // eslint-disable-next-line @typescript-eslint/no-unsafe-return
    return convertNullToUndefined(JSON.parse(body));
  }

  private prepareArgs(rawArgs: unknown[]) {
    return rawArgs.map((arg) => {
      if (arg instanceof Uint8Array) {
        return encode(arg as unknown as ArrayBuffer);
      } else if (typeof arg === 'function') {
        return this.callbacks.register(arg as unknown as () => void);
      } else {
        return arg;
      }
    });
  }
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
function convertNullToUndefined(input: any): any {
  if (input === null) {
    return undefined;
  }

  if (Array.isArray(input)) {
    return input.map(convertNullToUndefined);
  }

  if (typeof input === 'object') {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const result: { [key: string]: any } = {};

    for (const key in input) {
      if (Object.prototype.hasOwnProperty.call(input, key)) {
        // eslint-disable-next-line @typescript-eslint/no-unsafe-assignment, @typescript-eslint/no-unsafe-member-access
        result[key] = convertNullToUndefined(input[key]);
      }
    }

    return result;
  }

  return input;
}
