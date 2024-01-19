import { secretbox, randomBytes } from 'tweetnacl';
import {
  decodeBase64,
  decodeUTF8,
  encodeBase64,
  encodeUTF8,
} from 'tweetnacl-util';

export class Encryption {
  private key: Uint8Array;

  constructor(keyBase64: string) {
    this.key = decodeBase64(keyBase64);
  }

  encrypt(decrypted: string): string {
    const nonce = this.generateNonce();

    const decryptedUint8 = decodeUTF8(decrypted);

    const box = secretbox(decryptedUint8, nonce, this.key);

    const encrypted = new Uint8Array(nonce.length + box.length);
    encrypted.set(nonce);
    encrypted.set(box, nonce.length);

    const encryptedBase64 = encodeBase64(encrypted);

    return encryptedBase64;
  }

  generateNonce(): Uint8Array {
    return randomBytes(secretbox.nonceLength);
  }

  decrypt(encryptedBase64: string): string {
    const encrypted = decodeBase64(encryptedBase64);

    const nonce = encrypted.slice(0, secretbox.nonceLength);
    const box = encrypted.slice(secretbox.nonceLength, encrypted.length);

    const decrypted = secretbox.open(box, nonce, this.key);

    if (decrypted === null) {
      throw new Error('Could not decrypt message');
    }

    const message = encodeUTF8(decrypted);

    return message;
  }
}
