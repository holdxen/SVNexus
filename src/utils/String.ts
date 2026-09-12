export function trimStartSubstring(str: string, target: string) {
  while (target && str.startsWith(target)) {
    str = str.slice(target.length)
  }
  return str
}
