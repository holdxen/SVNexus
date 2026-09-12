import { OperationState } from '../../Operation'

export interface ChangesTreeViewStore {
  path: string
  operationState: OperationState
  refresh: () => Promise<void>
  setAddDialogVisible: (addDialogVisible: boolean) => void
}
