export async function mainWeb() {
  let hasOAuth2Token = false;

  try {
    hasOAuth2Token = localStorage.getItem('vaultOAuth2Token') !== null;
  } catch (
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    e: any
  ) {
    console.warn(`Failed to get oauth2 token from local storage: ${e}`);
  }

  if (document.location.pathname === '/' && !hasOAuth2Token) {
    await import('./mainWebUnauthenticated').then((mod) =>
      mod.mainUnauthenticated(),
    );
  } else {
    await import('./mainWebAuthenticated')
      .then((mod) => mod.mainAuthenticated())
      .catch(async (err) => {
        console.warn('Main loading error, falling back to landing page.', err);

        await import('./mainWebNotSupported').then((mod) =>
          mod.mainNotSupported(),
        );
      });
  }
}
