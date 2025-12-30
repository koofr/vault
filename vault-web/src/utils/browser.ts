export const isMacOS =
  // eslint-disable-next-line @typescript-eslint/no-explicit-any, @typescript-eslint/no-unsafe-member-access
  (navigator as any).userAgentData != null
    ? // eslint-disable-next-line @typescript-eslint/no-unsafe-member-access, @typescript-eslint/no-explicit-any
      (navigator as any).userAgentData.platform === 'macOS'
    : navigator.platform.indexOf('Mac') !== -1;
