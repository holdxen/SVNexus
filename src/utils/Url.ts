export function combineUrl(prefix: string, path: string) {
  if (prefix.endsWith('/')) {
    prefix = prefix.slice(0, -1)
  }
  if (path.startsWith('/')) {
    path = path.slice(1)
  }

  return path === '' ? prefix : `${prefix}/${path}`
}
