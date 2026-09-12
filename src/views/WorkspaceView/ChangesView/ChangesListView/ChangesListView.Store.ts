// import { invoke } from '@tauri-apps/api/core'
// import { create } from 'zustand'

// import { StatusEntry } from '../../../../bindings/StatusEntry'
// import { StatusOptions } from '../../../../bindings/StatusOptions'
// import { StatusResult } from '../../../../bindings/StatusResult'
// import itemSelection, { ItemSelection } from '../../../../utils/selection/immutable'
// import { defaultOperationState, OperationState } from '../../Operation'

// export interface ChangesListViewStore {
//   path: string
//   entries: StatusEntry[]
//   selection: ItemSelection<StatusEntry>
//   addDialogVisible: boolean
//   operationState: OperationState
//   setSelection: (selection: ItemSelection<StatusEntry>) => void
//   refresh: () => Promise<void>
//   setAddDialogVisible: (addDialogVisible: boolean) => void
// }

// export const createChangesListViewStore = (subversion: number, path: string) => {
//   const selectionChanged = (
//     selection: ItemSelection<StatusEntry>,
//     state: OperationState,
//   ): OperationState => {
//     let unversioned = 0
//     for (let i of selection.get()) {
//       if (i.node_status === 'Unversioned') {
//         unversioned += 1
//       }
//     }

//     return {
//       ...state,
//       refresh: true,
//       add: unversioned === selection.get().length,
//       revert: selection.get().length > 0,
//     }
//   }

//   return create<ChangesListViewStore>()((set) => {
//     return {
//       path: path,
//       entries: [],
//       selection: itemSelection<StatusEntry>([]),
//       operationState: {
//         ...defaultOperationState,
//         refresh: true,
//       },
//       addDialogVisible: false,
//       setSelection: (selection) => {
//         let unversioned = 0
//         for (let i of selection.get()) {
//           if (i.node_status === 'Unversioned') {
//             unversioned += 1
//           }
//         }
//         set((state) => ({
//           selection: selection,
//           operationState: selectionChanged(selection, state.operationState),
//         }))
//       },
//       async refresh() {
//         const options: StatusOptions = {
//           path: path,
//           revision: 'Working',
//           depth: 'Infinity',
//           get_all: false,
//           check_out_of_date: false,
//           check_working_copy: true,
//           no_ignore: false,
//           ignore_externals: false,
//           depth_as_sticky: false,
//           changelist: null,
//         }
//         try {
//           let result = await invoke<StatusResult>('subversion_status', {
//             id: subversion,
//             options: options,
//           })
//           set((state) => ({
//             entries: result.entries,
//             selection: itemSelection<StatusEntry>(result.entries),
//             operationState: selectionChanged(itemSelection([]), state.operationState),
//           }))
//           console.log('status: path=', path, ' result=', result)
//         } catch (error: any) {
//           console.log('Failed to status: ', error)
//         }
//       },
//       setAddDialogVisible(addDialogVisible) {
//         set({
//           addDialogVisible: addDialogVisible,
//         })
//       },
//     }
//   })
// }
