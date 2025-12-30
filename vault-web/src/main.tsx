import 'normalize.css';

if (import.meta.env.VITE_VAULT_APP === 'desktop') {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  import('./mainDesktop').then((mod) => mod.mainDesktop());
} else {
  // eslint-disable-next-line @typescript-eslint/no-floating-promises
  import('./mainWeb').then((mod) => mod.mainWeb());
}
