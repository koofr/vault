import { v4 as uuidv4 } from 'uuid';

export class Callbacks {
  callbacks: Map<string, (subscriptionId: number) => void>;

  constructor() {
    this.callbacks = new Map();
  }

  register(callback: (subscriptionId: number) => void): string {
    const callbackId = uuidv4();

    this.callbacks.set(callbackId, callback);

    return callbackId;
  }

  onCallback(callbackId: string, subscriptionId: number) {
    const callback = this.callbacks.get(callbackId);

    if (callback !== undefined) {
      callback(subscriptionId);
    }
  }
}
