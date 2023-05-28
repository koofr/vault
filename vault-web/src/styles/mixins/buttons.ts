import { css } from '@emotion/css';

import { allStates } from './hover';

export const buttonReset = `background: none;
border: none;
appearance: none;
cursor: pointer;
padding: 0;

${allStates} {
  outline: 0;
  text-decoration: none;
}`;

export const buttonHoverTransition = `background-color 0.2s ease-in-out, border-color 0.2s ease-in-out, color 0.2s ease-in-out`;

export const buttonStyle = (
  bgColor: string,
  borderColor: string,
  textColor: string,
  hoverBgColor: string,
  hoverBorderColor: string,
  hoverTextColor: string
) => css`
  background-color: ${bgColor};
  border-color: ${borderColor};

  ${allStates} {
    color: ${textColor};
  }

  &:hover,
  &:focus {
    background-color: ${hoverBgColor};
    border-color: ${hoverBorderColor};
    color: ${hoverTextColor};
  }
`;

export const buttonInlineStyle = (
  textColor: string,
  hoverTextColor: string
) => css`
  background: none;
  border: none;

  ${allStates} {
    color: ${textColor};
  }

  &:hover {
    color: ${hoverTextColor};
  }
`;
