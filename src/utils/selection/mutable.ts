import { ItemSelection } from './immutable'

class MutableItemSelection<T> extends ItemSelection<T> {
  override set(selection: number[], lastIndex: number | null): this {
    this.selection = selection
    this.lastIndex = lastIndex
    return this
  }
}

export default function mutableItemSelection<T>(
  items: T[],
  selection: number[] = [],
  lastIndex: number | null = null,
): MutableItemSelection<T> {
  return new MutableItemSelection(items, selection, lastIndex)
}

export { MutableItemSelection }
