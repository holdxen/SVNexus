export function isPromise<T>(value: unknown): value is Promise<T> {
  return value != null && typeof (value as Promise<T>).then === 'function'
}
