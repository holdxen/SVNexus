import { WorkspaceItem } from '@/bindings/WorkspaceItem'
export function workspaceItemStar(item: WorkspaceItem): boolean {
  if ('workingCopy' in item) {
    return item.workingCopy.star
  } else if ('repository' in item) {
    return item.repository.star
  } else {
    throw new Error('Unexpected workspace item')
  }
}
export function workspaceItemIdentity(item: WorkspaceItem): string {
  if ('workingCopy' in item) {
    return item.workingCopy.identity
  } else if ('repository' in item) {
    return item.repository.identity
  } else {
    throw new Error('Unexpected workspace item')
  }
}

type WorkingCopy = Exclude<WorkspaceItem, { repository: unknown }>['workingCopy']

export function workspaceItemGetWorkingCopy(item: WorkspaceItem): WorkingCopy {
  if ('workingCopy' in item) {
    return item.workingCopy
  } else {
    throw new Error('Unexpected workspace item type')
  }
}
