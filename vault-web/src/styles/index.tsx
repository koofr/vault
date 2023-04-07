import { css as reactCss } from '@emotion/react';

// we need to use @emotion/react css inside styled, but css is already used for
// inline styles, so this helper renames css
export function withReactCss<T>(f: (css: typeof reactCss) => T): T {
  return f(reactCss);
}
