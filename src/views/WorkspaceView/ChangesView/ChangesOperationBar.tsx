// import OperationAddIcon from '@icons/OperationAdd.svg?react'
// import OperationDiffIcon from '@icons/OperationDiff.svg?react'
// import OperationPatchIcon from '@icons/OperationPatch.svg?react'
// import OperationRevertIcon from '@icons/OperationRevert.svg?react'
// import OperationUpdateIcon from '@icons/OperationUpdate.svg?react'
// import RefreshIcon from '@icons/Refresh.svg?react'
// import { css, cx } from '@linaria/core'
// import { MouseEventHandler } from 'react'

// import OperationBar, { OperationIconProps } from '@/components/OperationBar'

// import { IconButton } from '../../../icons/IconButton'
// import { flex, flex_1, gap_x_1, py_1 } from '../../../styles/style'
// import { OperationState } from '../Operation'
// import { useChangesListViewContext } from './ChangesListView/ChangesListView.Context'

// const svg = css`
//   svg {
//     width: 20px;
//     height: 20px;
//   }
// `

// export interface ChangesOperationBarProps {
//   onRefresh?: () => void;
//   onAdd?: () => void
//   state?: OperationState
// }

// export default function ChangesOperationBar() {
//   const context = useChangesListViewContext()
//   const state = context.store((status) => status.operationState)

//   const setAddDialogVisible = context.store((status) => status.setAddDialogVisible)

//   const icons: OperationIconProps[] = [
//     {
//       sync: false,
//       tooltip: 'Refresh',
//       onClick: context.store((status) => status.refresh),
//       enable: state.refresh,
//       children: <RefreshIcon></RefreshIcon>,
//     },
//     {
//       tooltip: 'Add',
//       onClick: () => setAddDialogVisible(true),
//       enable: state.add,
//       children: <OperationAddIcon></OperationAddIcon>,
//     },
//     {
//       tooltip: 'Update',
//       enable: state.update,
//       children: <OperationUpdateIcon></OperationUpdateIcon>,
//     },
//     {
//       tooltip: 'Revert',
//       enable: state.revert,
//       children: <OperationRevertIcon></OperationRevertIcon>,
//     },
//     {
//       tooltip: 'Diff',
//       enable: state.diff,
//       children: <OperationDiffIcon></OperationDiffIcon>,
//     },
//     {
//       tooltip: 'Patch',
//       enable: state.patch,
//       children: <OperationPatchIcon></OperationPatchIcon>,
//     },
//   ]
//   return <OperationBar className={cx(gap_x_1, flex_1)} size={25} icons={icons}></OperationBar>
// }
