import { Global, css } from '@emotion/react';
import { useTheme } from '@emotion/react';
import { memo } from 'react';

import openSansV34LatinExtLatin300Woff from '../assets/fonts/open-sans-v34-latin-ext_latin-300.woff';
import openSansV34LatinExtLatin300Woff2 from '../assets/fonts/open-sans-v34-latin-ext_latin-300.woff2';
import openSansV34LatinExtLatin600Woff from '../assets/fonts/open-sans-v34-latin-ext_latin-600.woff';
import openSansV34LatinExtLatin600Woff2 from '../assets/fonts/open-sans-v34-latin-ext_latin-600.woff2';
import openSansV34LatinExtLatin700Woff from '../assets/fonts/open-sans-v34-latin-ext_latin-700.woff';
import openSansV34LatinExtLatin700Woff2 from '../assets/fonts/open-sans-v34-latin-ext_latin-700.woff2';
import openSansV34LatinExtLatin800Woff from '../assets/fonts/open-sans-v34-latin-ext_latin-800.woff';
import openSansV34LatinExtLatin800Woff2 from '../assets/fonts/open-sans-v34-latin-ext_latin-800.woff2';
import openSansV34LatinExtLatinRegularWoff from '../assets/fonts/open-sans-v34-latin-ext_latin-regular.woff';
import openSansV34LatinExtLatinRegularWoff2 from '../assets/fonts/open-sans-v34-latin-ext_latin-regular.woff2';

export const GlobalStyles = memo(() => {
  const theme = useTheme();

  return (
    <Global
      styles={css`
        /* open-sans-300 - latin-ext_latin */
        @font-face {
          font-family: 'Open Sans';
          font-style: normal;
          font-weight: 300;
          src: local(''),
            /* Chrome 26+, Opera 23+, Firefox 39+ */
              url(${openSansV34LatinExtLatin300Woff2}) format('woff2'),
            /* Chrome 6+, Firefox 3.6+, IE 9+, Safari 5.1+ */
              url(${openSansV34LatinExtLatin300Woff}) format('woff');
        }
        /* open-sans-regular - latin-ext_latin */
        @font-face {
          font-family: 'Open Sans';
          font-style: normal;
          font-weight: 400;
          src: local(''),
            /* Chrome 26+, Opera 23+, Firefox 39+ */
              url(${openSansV34LatinExtLatinRegularWoff2}) format('woff2'),
            /* Chrome 6+, Firefox 3.6+, IE 9+, Safari 5.1+ */
              url(${openSansV34LatinExtLatinRegularWoff}) format('woff');
        }
        /* open-sans-600 - latin-ext_latin */
        @font-face {
          font-family: 'Open Sans';
          font-style: normal;
          font-weight: 600;
          src: local(''),
            /* Chrome 26+, Opera 23+, Firefox 39+ */
              url(${openSansV34LatinExtLatin600Woff2}) format('woff2'),
            /* Chrome 6+, Firefox 3.6+, IE 9+, Safari 5.1+ */
              url(${openSansV34LatinExtLatin600Woff}) format('woff');
        }
        /* open-sans-700 - latin-ext_latin */
        @font-face {
          font-family: 'Open Sans';
          font-style: normal;
          font-weight: 700;
          src: local(''),
            /* Chrome 26+, Opera 23+, Firefox 39+ */
              url(${openSansV34LatinExtLatin700Woff2}) format('woff2'),
            /* Chrome 6+, Firefox 3.6+, IE 9+, Safari 5.1+ */
              url(${openSansV34LatinExtLatin700Woff}) format('woff');
        }
        /* open-sans-800 - latin-ext_latin */
        @font-face {
          font-family: 'Open Sans';
          font-style: normal;
          font-weight: 800;
          src: local(''),
            /* Chrome 26+, Opera 23+, Firefox 39+ */
              url(${openSansV34LatinExtLatin800Woff2}) format('woff2'),
            /* Chrome 6+, Firefox 3.6+, IE 9+, Safari 5.1+ */
              url(${openSansV34LatinExtLatin800Woff}) format('woff');
        }

        * {
          box-sizing: border-box;
        }
        *:before,
        *:after {
          box-sizing: border-box;
        }

        html {
          font-size: ${theme.fontSizeBase};
          height: 100%;
          -webkit-tap-highlight-color: rgba(0, 0, 0, 0);
        }

        body {
          font-family: ${theme.fontFamilyBase};
          font-size: ${theme.fontSizeBase};
          line-height: ${theme.lineHeightBase};
          color: ${theme.colors.text};
          font-weight: normal;
          background-color: #fff;
          overflow: auto;
          min-height: 100%;
          display: flex;
          flex-direction: column;
          flex-grow: 1;
        }

        #root {
          display: flex;
          flex-direction: column;
          flex-grow: 1;
        }

        input,
        button,
        select,
        textarea {
          font-family: inherit;
          font-size: inherit;
          line-height: inherit;
        }

        a {
          color: ${theme.colors.link};
          text-decoration: none;

          &:hover,
          &:focus {
            color: ${theme.colors.linkHover};
            text-decoration: ${theme.linkHoverDecoration};
          }
        }

        input:focus,
        textarea:focus,
        keygen:focus,
        select:focus {
          outline: none;
        }

        img {
          vertical-align: middle;
        }

        // iOS "clickable elements" fix for role="button"
        //
        // Fixes "clickability" issue (and more generally, the firing of events such as focus as well)
        // for traditionally non-focusable elements with role="button"
        // see https://developer.mozilla.org/en-US/docs/Web/Events/click#Safari_Mobile

        [role='button'] {
          cursor: pointer;
        }
      `}
    />
  );
});
