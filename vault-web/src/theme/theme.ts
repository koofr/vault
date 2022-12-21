import { colors } from './brand';

export * from './brand';

// this is overriden dynamically
export const isMobile: boolean = false;

export const linkHoverDecoration = 'underline';
export const fontSizeBase = `14px`;
export const fontWeightNormal = 'normal';
export const fontWeightBold = '600';
export const lineHeightBase = '1.428571429'; // 20/14
export const lineHeightComputed = '20px'; // floor(fontSizeBase * lineHeightBase))

export const gutter = '25px';
export const gutterMobile = '15px';

export const boxShadow = `0 1px 3px 0 ${colors.boxShadow}`;

export const zindex = {
  modalBg: 1040,
  modal: 1050,

  navbarExtra: 600,
  navbarMain: 601,
  spaceUsage: 602,
  uploads: 605,
  dropZoneLines: 610,
  repoFilesAddMenu: 1000,
  dashboardMenu: 1006,
  notifications: 9000,
};
