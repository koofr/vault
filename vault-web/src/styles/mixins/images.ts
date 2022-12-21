import { css } from '@emotion/css';

export const imgRetinaBase = (
  file1x: string,
  file2x: string,
  width: number,
  height: number
) => {
  return css`
    background-image: url(${file1x});
    background-repeat: no-repeat;
    background-size: ${width}px ${height}px;

    @media only screen and (min-resolution: 192dpi),
      only screen and (min-resolution: 2dppx) {
      background-image: url(${file2x});
    }
  `;
};

export const imgRetina = (
  file1x: string,
  file2x: string,
  width: number,
  height: number
) => {
  return css`
    ${imgRetinaBase(file1x, file2x, width, height)}
    width: ${width}px;
    height: ${height}px;
  `;
};
