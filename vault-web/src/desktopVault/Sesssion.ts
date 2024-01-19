import { Callbacks } from './Callbacks';
import { RequestEncryption } from './RequestEncryption';

export interface SessionMessageStart {
  type: 'Start';
  sessionId: string;
}

export interface SessionMessageCallback {
  type: 'Callback';
  callbackId: string;
}

export type SessionMessage = SessionMessageStart | SessionMessageCallback;

export class Session {
  baseUrl: string;
  requestEncryption: RequestEncryption;
  callbacks: Callbacks;

  eventSource?: EventSource;
  sessionId?: string;
  connectResolve?: () => void;

  constructor(
    baseUrl: string,
    requestEncryption: RequestEncryption,
    callbacks: Callbacks,
  ) {
    this.baseUrl = baseUrl;
    this.requestEncryption = requestEncryption;
    this.callbacks = callbacks;

    this.eventSource = undefined;
    this.sessionId = undefined;
    this.connectResolve = undefined;
  }

  connect(): Promise<void> {
    this.sessionId = undefined;

    const encryptedRequest = this.requestEncryption.encryptRequest({
      method: 'GET',
      uri: '/session',
    });

    const res = new Promise<void>((resolve) => {
      this.connectResolve = resolve;
    });

    this.eventSource = new EventSource(
      `${this.baseUrl}/?req=${encodeURIComponent(encryptedRequest)}`,
    );

    this.eventSource.onmessage = this.onMessage;

    return res;
  }

  onMessage = (event: MessageEvent<any>) => {
    const message: SessionMessage = JSON.parse(event.data);

    if (message.type === 'Start') {
      this.onStart(message.sessionId);
    } else if (message.type === 'Callback') {
      this.onCallback(message.callbackId);
    }
  };

  onStart(sessionId: string) {
    this.sessionId = sessionId;
    this.requestEncryption.setSessionId(sessionId);

    this.connectResolve?.();
    this.connectResolve = undefined;
  }

  onCallback(callbackId: string) {
    this.callbacks.onCallback(callbackId);
  }
}
