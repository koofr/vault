import { invoke } from '@tauri-apps/api/tauri';
import ReactDOM from 'react-dom/client';

import { getApp } from './App';
import { Config } from './config';
import { Callbacks } from './desktopVault/Callbacks';
import { Encryption } from './desktopVault/Encryption';
import { RequestEncryption } from './desktopVault/RequestEncryption';
import { Session } from './desktopVault/Sesssion';
import { WebVaultClient } from './desktopVault/WebVaultClient';
import { createProxy } from './desktopVault/webVaultProxy';
import { LandingPageDesktop } from './pages/LandingPageDesktop';

export const mainDesktop = async () => {
  const isTauri = window.__TAURI_METADATA__ !== undefined;

  const configPromise = fetch('/config.json').then(
    (res) => res.json() as Promise<Config>,
  );

  const config = await configPromise;

  const desktopServerUrl = isTauri
    ? await invoke<string>('get_desktop_server_url')
    : `http://127.0.0.1:1421`;
  const appSecret = isTauri
    ? await invoke<string>('get_app_secret')
    : 'XrwBl00MUbeAZ4QBW2F+YDFBv80f2kes49VDx7wUs7Y=';

  const encryption = new Encryption(appSecret);

  const requestEncryption = new RequestEncryption(encryption);

  const callbacks = new Callbacks();

  const session = new Session(desktopServerUrl, requestEncryption, callbacks);

  await session.connect();

  const webVaultClient = new WebVaultClient(
    desktopServerUrl,
    requestEncryption,
    callbacks,
  );
  const webVault = createProxy(webVaultClient);

  const landingPage = (
    <LandingPageDesktop
      onLogin={() => {
        webVault.login();
      }}
    />
  );

  const app = getApp(config, webVault, landingPage);

  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    app,
  );
};
