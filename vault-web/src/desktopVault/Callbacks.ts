import { v4 as uuidv4 } from 'uuid';

export class Callbacks {
  callbacks: Map<string, Function>;

  constructor() {
    this.callbacks = new Map();
  }

  register(callback: Function): string {
    const callbackId = uuidv4();

    this.callbacks.set(callbackId, callback);

    return callbackId;
  }

  onCallback(callbackId: string) {
    const callback = this.callbacks.get(callbackId);

    if (callback !== undefined) {
      callback();
    }
  }
}
