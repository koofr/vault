import { encode, decode } from 'base64-arraybuffer';

import { Encryption } from './Encryption';

export interface RequestId {
  sessionId: string;
  sequenceId?: number;
}

export interface EncryptedRequest {
  id?: RequestId;
  method: string;
  uri: string;
  headers: Record<string, string>;
  body: string;
}

export interface EncryptedResponse {
  headers: Record<string, string>;
  body: string;
}

export class RequestEncryption {
  encryption: Encryption;

  sessionId?: string;
  sequenceId: number;

  constructor(encryption: Encryption) {
    this.encryption = encryption;

    this.sessionId = undefined;
    this.sequenceId = 0;
  }

  setSessionId(sessionId: string) {
    this.sessionId = sessionId;
    this.sequenceId = 0;
  }

  nextSequenceId(): number {
    const sequenceId = this.sequenceId;

    this.sequenceId++;

    return sequenceId;
  }

  encryptRequest(opts: {
    method: string;
    uri: string;
    headers?: Record<string, string>;
    body?: string;
  }): string {
    const id: RequestId | undefined =
      this.sessionId !== undefined
        ? {
            sessionId: this.sessionId,
            sequenceId: this.nextSequenceId(),
          }
        : undefined;

    const encryptedRequest: EncryptedRequest = {
      id,
      method: opts.method,
      uri: opts.uri,
      headers: opts.headers ?? {},
      body: encode(new TextEncoder().encode(opts.body ?? '')),
    };

    const encryptedRequestJSON = JSON.stringify(encryptedRequest);

    return this.encryption.encrypt(encryptedRequestJSON);
  }

  decryptResponse(encryptedResponse: string): {
    headers: Record<string, string>;
    body: string;
  } {
    const decryptedResponseJSON = this.encryption.decrypt(encryptedResponse);

    const response: EncryptedResponse = JSON.parse(decryptedResponseJSON);

    return {
      headers: response.headers,
      body: new TextDecoder().decode(decode(response.body)),
    };
  }
}
