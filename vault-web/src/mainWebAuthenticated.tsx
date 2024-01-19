import ReactDOM from 'react-dom/client';
import streamSaver from 'streamsaver';

import { getApp } from './App';
import { Config } from './config';
import { LandingPageLazy } from './pages/LandingPageLazy';
import init, { WebVault, initConsole } from './vault-wasm/vault-wasm';
import { BrowserEventstreamWebSocketDelegateImpl } from './webVault/BrowserEventstreamWebSocketDelegateImpl';
import { BrowserHttpClientDelegateImpl } from './webVault/BrowserHttpClientDelegateImpl';

export const mainAuthenticated = async () => {
  const configPromise = fetch('/config.json').then(
    (res) => res.json() as Promise<Config>,
  );

  await init();
  initConsole();

  streamSaver.mitm =
    window.location.origin +
    '/streamsaver-2.0.6-34ea69e/mitm.html?version=2.0.0';

  const config = await configPromise;

  const baseUrl = config.baseUrl;
  const oauth2ClientId = config.oauth2ClientId;
  const oauth2ClientSecret = config.oauth2ClientSecret;
  const oauth2RedirectUri = window.location.origin + '/oauth2callback';

  const webVault = new WebVault(
    baseUrl,
    baseUrl,
    oauth2ClientId,
    oauth2ClientSecret,
    oauth2RedirectUri,
    new BrowserHttpClientDelegateImpl(),
    new BrowserEventstreamWebSocketDelegateImpl(),
    localStorage,
  );

  // don't load the app on oauth2 login or logout
  if (document.location.pathname !== '/oauth2callback') {
    webVault.load();
  }

  const landingPage = <LandingPageLazy />;

  const app = getApp(config, webVault, landingPage);

  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
    app,
  );
};
