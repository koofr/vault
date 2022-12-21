import { memo, useLayoutEffect } from 'react';

export const RemoveAppLoading = memo(() => {
  useLayoutEffect(() => {
    const el = document.getElementById('app-loading');

    if (el == null) {
      return;
    }

    el.classList.add('is-hidden');

    setTimeout(() => {
      el.parentNode!.removeChild(el);
    }, 300);
  }, []);

  return null;
});
