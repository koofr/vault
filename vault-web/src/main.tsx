import 'normalize.css';

if (import.meta.env.VITE_VAULT_APP === 'desktop') {
  import('./mainDesktop').then((mod) => mod.mainDesktop());
} else {
  import('./mainWeb').then((mod) => mod.mainWeb());
}
