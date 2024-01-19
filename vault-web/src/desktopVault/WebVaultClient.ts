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

  getUrl(name: string, params: { [key: string]: any }): string {
    const encryptedRequest = this.requestEncryption.encryptRequest({
      method: 'GET',
      uri: `/WebVault/${name}?${new URLSearchParams(params).toString()}`,
    });

    return `${this.baseUrl}/?req=${encodeURIComponent(encryptedRequest)}`;
  }

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
          reject(request);
        };

        request.send(requestBody);
      });
    } else {
      request.send(requestBody);

      return this.handleResponse(request, bytes);
    }
  }

  handleResponse(request: XMLHttpRequest, bytes: boolean) {
    const { body } = this.requestEncryption.decryptResponse(
      request.responseText,
    );

    if (request.status !== 200 && request.status !== 204) {
      throw new Error(`Call error: ${request.status}: ${body}`);
    }

    if (request.status === 204) {
      return undefined;
    }

    if (bytes) {
      return new Uint8Array(decode(body));
    }

    if (body === '') {
      return undefined;
    }

    return convertNullToUndefined(JSON.parse(body));
  }

  private prepareArgs(rawArgs: any[]) {
    return rawArgs.map((arg) => {
      if (arg instanceof Uint8Array) {
        return encode(arg);
      } else if (typeof arg === 'function') {
        return this.callbacks.register(arg);
      } else {
        return arg;
      }
    });
  }
}

function convertNullToUndefined(input: any): any {
  if (input === null) {
    return undefined;
  }

  if (Array.isArray(input)) {
    return input.map(convertNullToUndefined);
  }

  if (typeof input === 'object') {
    const result: { [key: string]: any } = {};

    for (const key in input) {
      if (input.hasOwnProperty(key)) {
        result[key] = convertNullToUndefined(input[key]);
      }
    }

    return result;
  }

  return input;
}
