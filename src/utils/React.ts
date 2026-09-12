import { type DependencyList, type EffectCallback, useEffect, useRef } from 'react'

type ChangedDeps<T extends readonly unknown[]> = {
  readonly [K in keyof T]: boolean
}

type PreviousDeps<T extends readonly unknown[]> = T | null

/**
 * 类似 React.useEffect，但会额外告诉你：
 *
 * 1. 哪些 dependency 发生了变化
 * 2. 上一次的 dependency 值
 *
 * @example
 *
 * useChangedEffect(
 *   ([userChanged, pageChanged], previous) => {
 *     if (userChanged) {
 *       console.log(
 *         previous?.[0],
 *         '->',
 *         userId,
 *       );
 *     }
 *
 *     if (pageChanged) {
 *       console.log(
 *         previous?.[1],
 *         '->',
 *         page,
 *       );
 *     }
 *   },
 *   [userId, page],
 * );
 */
export function useChangedEffect<const T extends readonly unknown[]>(
  effect: (changed: ChangedDeps<T>, previous: PreviousDeps<T>) => ReturnType<EffectCallback>,
  deps: T,
): void {
  const previousDepsRef = useRef<T | null>(null)

  useEffect(() => {
    const previous = previousDepsRef.current

    const changed = deps.map((dep, index) => {
      // 首次执行没有 previous dependency，
      // 所以认为没有任何 dependency 发生“变化”。
      if (previous === null) {
        return true
      }

      // 和 React useEffect 一致，使用 Object.is 比较。
      return !Object.is(dep, previous[index])
    }) as ChangedDeps<T>

    // 保存当前 dependency，
    // 供下一次 effect 执行时比较。
    previousDepsRef.current = deps

    return effect(changed, previous)
  }, deps as DependencyList)
}
