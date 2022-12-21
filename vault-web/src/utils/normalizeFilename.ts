// we need to normalize filenames on macOS (NFD -> NFC)
export function normalizeFilename(name: string): string {
  if (!String.prototype.normalize) {
    return name;
  }

  const nfc = name.normalize();

  const normalizedName = nfc
    .split('')
    .filter((c) => c >= '\u0020' && !(c >= '\u007f' && c <= '\u009f'))
    .join('');

  return normalizedName;
}
