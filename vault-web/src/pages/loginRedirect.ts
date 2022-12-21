export const loginRedirectKey = 'vaultLoginRedirect';

export const getLoginRedirect = (): string | undefined => {
  try {
    return localStorage.getItem(loginRedirectKey) ?? undefined;
  } catch {
    return undefined;
  }
};

export const setLoginRedirect = (redirect: string) => {
  try {
    localStorage.setItem(loginRedirectKey, redirect);
  } catch {}
};

export const removeLoginRedirect = () => {
  try {
    localStorage.removeItem(loginRedirectKey);
  } catch {}
};
