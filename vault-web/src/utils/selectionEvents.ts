import { isMacOS } from './browser';

export const isExtend = (event: { ctrlKey: boolean; metaKey: boolean }) =>
  isMacOS ? event.metaKey : event.ctrlKey;

export const isRange = (event: { shiftKey: boolean }) => event.shiftKey;
