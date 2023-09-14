import 'normalize.css';

let hasOAuth2Token = false;

try {
  hasOAuth2Token = localStorage.getItem('vaultOAuth2Token') !== null;
} catch (e) {
  console.warn(`Failed to get oauth2 token from local storage: ${e}`);
}

if (document.location.pathname === '/' && !hasOAuth2Token) {
  import('./mainUnauthenticated').then((mod) => mod.mainUnauthenticated());
} else {
  import('./mainAuthenticated')
    .then((mod) => mod.mainAuthenticated())
    .catch((err) => {
      console.warn('Main loading error, falling back to landing page.', err);

      import('./mainNotSupported').then((mod) => mod.mainNotSupported());
    });
}
