export function selectRange(
  el: HTMLInputElement,
  start: number,
  end: number
): void {
  if (el.setSelectionRange) {
    el.focus();
    el.setSelectionRange(start, end);
  } else if ((el as any).createTextRange) {
    const range = (el as any).createTextRange();
    range.collapse(true);
    range.moveEnd('character', end);
    range.moveStart('character', start);
    range.select();
  }
}

export function selectFilenameRange(
  el: HTMLInputElement,
  isDir: boolean
): void {
  const name = el.value;
  const nameParts = name.split('.');

  const start = 0;
  let end = name.length;

  if (!isDir && nameParts.length > 1) {
    const ext = nameParts[nameParts.length - 1];

    end -= ext.length + 1;
  }

  selectRange(el, start, end);
}
