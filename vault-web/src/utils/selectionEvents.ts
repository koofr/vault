const isMacOS =
  (navigator as any).userAgentData != null
    ? (navigator as any).userAgentData.platform === 'macOS'
    : navigator.platform.indexOf('Mac') !== -1;

export const isExtend = (event: { ctrlKey: boolean; metaKey: boolean }) =>
  isMacOS ? event.metaKey : event.ctrlKey;

export const isRange = (event: { shiftKey: boolean }) => event.shiftKey;
