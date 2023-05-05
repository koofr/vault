import keycode from 'keycode';

export function isEventKeys(
  event: KeyboardEvent,
  key: string,
  {
    altKey = false,
    ctrlKey = false,
    metaKey = false,
    shiftKey = false,
  }: {
    altKey?: boolean;
    ctrlKey?: boolean;
    metaKey?: boolean;
    shiftKey?: boolean;
  },
): boolean {
  return (
    keycode.isEventKey(event, key) &&
    event.altKey === altKey &&
    event.ctrlKey === ctrlKey &&
    event.metaKey === metaKey &&
    event.shiftKey === shiftKey
  );
}
