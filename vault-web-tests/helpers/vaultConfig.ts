import path from 'path';
import { readFileSync } from 'fs';

const configPath = path.join(
  import.meta.dirname,
  '../../vault-web/public/config.json',
);

// eslint-disable-next-line @typescript-eslint/no-unsafe-assignment
export const config: {
  baseUrl: string;
  oauth2ClientId: string;
  oauth2ClientSecret: string;
} = JSON.parse(readFileSync(configPath).toString('utf8'));

export const ignoreHTTPSErrors = /127.0.0.1|localhost/.test(config.baseUrl);
