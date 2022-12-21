export function isFocusedElementInput(): boolean {
  const activeElement = document.activeElement;
  const inputTags = ['input', 'select', 'button', 'textarea'];

  return (
    activeElement != null &&
    inputTags.indexOf(activeElement.tagName.toLowerCase()) !== -1
  );
}
