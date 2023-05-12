export function joinParentName(parentPath: string, name: string): string {
  if (parentPath === '/') {
    return `/${name}`;
  }
  return `${parentPath}/${name}`;
}

export function splitParentName(path: string): [string, string] {
  if (path === '/') {
    throw new Error('cannot call splitParentName for /');
  }
  const parts = path.split('/');
  const name = parts.pop();
  let parentPath = parts.join('/');
  if (parentPath === '') {
    parentPath = '/';
  }
  return [parentPath, name];
}
