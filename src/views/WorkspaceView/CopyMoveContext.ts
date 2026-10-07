import { createContext, useContext } from 'react'

import { CopySourceItem as BaseCopySourceItem } from '@/bindings/CopySourceItem'
import { NodeKind } from '@/bindings/NodeKind'

export interface CopySourceItem extends BaseCopySourceItem {
  nodeKind?: NodeKind
}

export interface MoveSourceItem {
  path: string
  nodeKind?: NodeKind
}

export type CopyMoveSource =
  | {
      copy: CopySourceItem[]
    }
  | {
      move: MoveSourceItem[]
      isLocal: boolean
    }

export interface CopyMoveItems {
  source?: CopyMoveSource
  setSource: (source?: CopyMoveSource) => void
}

export function canPatse(source?: CopyMoveSource) {
  if (typeof source === 'object') {
    if ('copy' in source) {
      return source.copy.length > 0
    } else if ('move' in source) {
      return source.move.length > 0
    }
  }
  return false
}

export const CopyMoveItemsContext = createContext<CopyMoveItems | null>(null)

export function useCopyMoveItems(): CopyMoveItems {
  const copyItems = useContext(CopyMoveItemsContext)

  if (copyItems === null) {
    throw new Error('No context provided')
  }

  return copyItems
}
