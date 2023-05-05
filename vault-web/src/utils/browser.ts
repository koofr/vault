export const isMacOS =
  (navigator as any).userAgentData != null
    ? (navigator as any).userAgentData.platform === 'macOS'
    : navigator.platform.indexOf('Mac') !== -1;
