import { readFileSync } from 'fs';
import path from 'path';

export const storageStatePath = path.join(
  __dirname,
  'playwright/.auth/user.json'
);

export let storageState: any;

try {
  storageState = JSON.parse(readFileSync(storageStatePath).toString('utf8'));
} catch {
  throw new Error(`Missing file ${storageStatePath}`);
}
