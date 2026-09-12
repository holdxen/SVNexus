function range(start: number, end: number): number[] {
  if (start > end) {
    ;[start, end] = [end, start]
  }

  const list: number[] = []
  for (let i = start; i <= end; i += 1) {
    list.push(i)
  }

  return list
}

const includes = <T>(arr: readonly T[], item: T): boolean => arr.indexOf(item) !== -1

const cmp = (a: number, b: number): number => {
  if (a > b) return 1
  if (a < b) return -1
  return 0
}

class ItemSelection<T> {
  public items: T[]
  public selection: number[]
  public lastIndex: number | null

  constructor(items: T[], selection: number[] = [], lastIndex: number | null = null) {
    if (!Array.isArray(items)) {
      throw new TypeError('Expected an array')
    }

    this.items = items
    this.selection = selection
    this.lastIndex = lastIndex
  }

  getIndices(): number[] {
    return this.selection.slice().sort(cmp)
  }

  get(): T[] {
    return this.getIndices().map((index) => this.items[index]!)
  }

  set(selection: number[], lastIndex: number | null): ItemSelection<T> {
    return new ItemSelection(this.items, selection, lastIndex)
  }

  isSelectedIndex(index: number): boolean {
    return includes(this.selection, index)
  }

  isSelected(item: T): boolean {
    return includes(this.get(), item)
  }

  clear(): ItemSelection<T> {
    return this.set([], null)
  }

  add(index: number): ItemSelection<T> {
    return this.set([...this.selection, index], this.lastIndex)
  }

  remove(index: number): ItemSelection<T> {
    return this.set(
      this.selection.filter((selectedIndex) => selectedIndex !== index),
      null,
    )
  }

  select(index: number): ItemSelection<T> {
    return this.set([index], index)
  }

  deselect(index: number): ItemSelection<T> {
    return this.remove(index)
  }

  selectRange(index: number, end: number | null = null): ItemSelection<T> {
    if (end !== null) {
      return this.set(range(index, end), null)
    }

    if (typeof this.lastIndex !== 'number') {
      return this.select(index)
    }

    return this.set(range(this.lastIndex, index), this.lastIndex)
  }

  selectToggle(index: number): ItemSelection<T> {
    if (this.isSelectedIndex(index)) {
      return this.remove(index)
    }

    return this.set([...this.selection, index], index)
  }

  selectAll(): ItemSelection<T> {
    return this.set(
      this.items.map((_item, index) => index),
      null,
    )
  }
}

export default function itemSelection<T>(
  items: T[],
  selection: number[] = [],
  lastIndex: number | null = null,
): ItemSelection<T> {
  return new ItemSelection(items, selection, lastIndex)
}

export { ItemSelection }
