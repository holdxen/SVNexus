/**
 * LimitedDictionary - 有容量上限的字典
 *
 * 当元素数量达到 Limit 时，添加新元素会淘汰最早插入的一个（类似简单的 FIFO 缓存）。
 *
 * 说明：
 * - 使用 Map 而不是普通对象，因为 Map 能保证保留插入顺序，
 *   这与原 C# 代码依赖 Dictionary.First()（.NET 的 Dictionary 在未发生删除操作时
 *   通常按插入顺序遍历）想要表达的“淘汰最早插入项”的语义是一致的，且行为更可预期。
 * - TKey 对应 TypeScript 中的 key 类型（需保证是 Map 可用作 key 的类型，
 *   如 string / number / object 引用等）。
 */
export class LimitedDictionary<TKey, TValue> {
  public readonly dictionary: Map<TKey, TValue> = new Map()
  public readonly limit: number = 0

  constructor(limit: number, dictionary: Map<TKey, TValue> = new Map()) {
    this.limit = limit
    this.dictionary = dictionary
  }

  public clone(): LimitedDictionary<TKey, TValue> {
    return new LimitedDictionary(this.limit, new Map(this.dictionary))
  }

  public clear(): void {
    this.dictionary.clear()
  }

  /**
   * 添加或更新一个键值对。
   * 如果 limit <= 0，则清空字典并返回 null（相当于禁用缓存）。
   * 添加后如果元素数量达到 limit，则淘汰并返回最早插入的键值对；否则返回 null。
   */
  public add(key: TKey, value: TValue): [TKey, TValue] | null {
    if (this.limit <= 0) {
      this.dictionary.clear()
      return null
    }

    this.dictionary.set(key, value)

    if (this.dictionary.size < this.limit) {
      return null
    }

    const firstEntry = this.dictionary.entries().next()
    if (firstEntry.done) {
      return null
    }

    const [firstKey, firstValue] = firstEntry.value
    this.dictionary.delete(firstKey)
    return [firstKey, firstValue]
  }

  public tryGetValue(key: TKey): TValue | undefined {
    return this.dictionary.get(key)
  }

  public remove(key: TKey): boolean {
    return this.dictionary.delete(key)
  }

  public get(key: TKey): TValue {
    const value = this.dictionary.get(key)
    if (value === undefined && !this.dictionary.has(key)) {
      throw new Error(`Key not found: ${String(key)}`)
    }
    return value as TValue
  }

  public set(key: TKey, value: TValue): void {
    this.dictionary.set(key, value)
  }
}
