import { Toast } from '@douyinfe/semi-ui'
import OperationAddIcon from '@icons/OperationAdd.svg?react'
import OperationCommitIcon from '@icons/OperationCommit.svg?react'
import OperationCopyIcon from '@icons/OperationCopy.svg?react'
import OperationDeleteIcon from '@icons/OperationDelete.svg?react'
import OperationDiffIcon from '@icons/OperationDiff.svg?react'
import OperationExportIcon from '@icons/OperationExport.svg?react'
import OperationInfoIcon from '@icons/OperationInfo.svg?react'
import OperationLockIcon from '@icons/OperationLock.svg?react'
import OperationMergeIcon from '@icons/OperationMerge.svg?react'
import OperationMkdirIcon from '@icons/OperationMkdir.svg?react'
import OperationMoveIcon from '@icons/OperationMove.svg?react'
import OperationPatchIcon from '@icons/OperationPatch.svg?react'
import OperationRevertIcon from '@icons/OperationRevert.svg?react'
import OperationSwitchIcon from '@icons/OperationSwitch.svg?react'
import OperationUnlockIcon from '@icons/OperationUnlock.svg?react'
import OperationUpdateIcon from '@icons/OperationUpdate.svg?react'
import RefreshIcon from '@icons/Refresh.svg?react'
import { cx } from '@linaria/core'
import { useMemoizedFn } from 'ahooks'
import React, { useEffect, useState } from 'react'
import { createPortal } from 'react-dom'

import { InfoOptions } from '@/bindings/InfoOptions'
import { NodePropertyName } from '@/bindings/NodePropertyName'
import { PropertyGetOptions } from '@/bindings/PropertyGetOptions'
import { PropertySetOptions } from '@/bindings/PropertySetOptions'
import { StatusOptions } from '@/bindings/StatusOptions'
import { ContextMenu, ContextMenuItemModel } from '@/components/ContextMenu'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import VirtualList from '@/components/VirtualList'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useModal } from '@/lib/multi-modal'
import {
  hidden,
  min_w_0,
  overflow_hidden,
  whitespace_nowrap,
  flex,
  items_center,
  gap_x_1,
  flex_1,
  flex_wrap,
} from '@/styles/Classes'
import { list_item, list_item_selected } from '@/styles/Components'
import errorHumanString from '@/utils/Error'
import { repoPath } from '@/utils/Path'
import itemSelection, { ItemSelection } from '@/utils/selection/immutable'

import { StatusEntry } from '../../../../bindings/StatusEntry'
import { defaultOperationState, OperationState } from '../../Operation'
import { fromStatusEntry, WorkingCopyItem } from '../../WorkingCopyItem'
import { useWorkingCopyContext } from '../../WorkingCopyView'
import { useWorkspaceContext } from '../../WorkspaceView'
import OperationHandler from '../OperationHandler'

export interface ChangesListViewProps {
  visible: boolean
  onSelected?: (entry: string | null) => void
  onRefresh?: () => void
}

export function ChangesListView({ visible, onSelected, onRefresh }: ChangesListViewProps) {
  const workingCopy = useWorkingCopyContext()
  const workspace = useWorkspaceContext()
  const subversion = useSubversion()

  const [entries, setEntries] = useState<StatusEntry[]>([])
  const [selection, setSelection] = useState(itemSelection<StatusEntry>([]))

  useEffect(() => {
    if (selection.get().length === 1) {
      onSelected?.(selection.get()[0].path)
    } else if (selection.get().length === 0) {
      onSelected?.(null)
    }
  }, [selection])

  const displayItems = () => {
    return selection.get().map((e) => {
      const model = fromStatusEntry(e, false, workingCopy.path)
      return { ...model, path: e.path }
    })
  }

  const refresh = async () => {
    const options: StatusOptions = {
      path: workingCopy.path,
      revision: 'working',
      depth: 'infinity',
      getAll: false,
      checkOutOfDate: false,
      checkWorkingCopy: true,
      noIgnore: false,
      ignoreExternals: false,
      depthAsSticky: false,
      changelist: null,
    }
    await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        const result = await context.status(options)
        setEntries(result.entries)
        setSelection(itemSelection(result.entries))
      },
      onError: (error: any) => {
        Toast.error({
          content: `Failed to get status: ${errorHumanString(error)}`,
          stack: true,
        })
      },
    })
    onRefresh?.()
  }
  useEffect(() => {
    refresh()
  }, [])

  const modal = useModal()

  const operationHandler = new OperationHandler(modal, refresh)

  const selectionChanged = (
    selection: ItemSelection<StatusEntry>,
    state: OperationState,
  ): OperationState => {
    let unversioned = 0
    for (let i of selection.get()) {
      if (i.nodeStatus === 'unversioned') {
        unversioned += 1
      }
    }

    const length = selection.get().length
    const selectedEntries = selection.get()

    return {
      ...state,
      refresh: true,
      patch:
        length === 1 &&
        selectedEntries[0].nodeKind === 'directory' &&
        selectedEntries[0].nodeStatus !== 'unversioned',
      update:
        length > 0 &&
        selection.get().every((e) => e.nodeStatus !== 'unversioned' && e.nodeStatus !== 'added'),
      add: unversioned === selection.get().length && length > 0,
      revert: selection.get().length > 0,
      diff: length == 1 && selection.get()[0].nodeStatus !== 'unversioned',
      commit: length > 0 && selection.get().every((i) => i.nodeStatus !== 'unversioned'),
      lock:
        length > 0 &&
        selectedEntries.every(
          (i) =>
            i.lock === null &&
            i.nodeKind !== 'directory' &&
            i.nodeStatus !== 'added' &&
            i.nodeStatus !== 'unversioned',
        ),
      unlock:
        length > 0 &&
        selectedEntries.every(
          (i) =>
            i.lock !== null &&
            i.nodeKind !== 'directory' &&
            i.nodeStatus !== 'added' &&
            i.nodeStatus !== 'unversioned',
        ),
      delete: length > 0,
      info: length === 1,
      mkdir:
        length === 1 &&
        selectedEntries[0].nodeKind === 'directory' &&
        selectedEntries[0].nodeStatus !== 'unversioned',
      export:
        length === 1 &&
        selectedEntries[0].nodeKind === 'directory' &&
        selectedEntries[0].nodeStatus !== 'unversioned' &&
        selectedEntries[0].nodeStatus !== 'added',
      ignore: length === 1,
    }
  }

  const state = selectionChanged(selection, {
    ...defaultOperationState,
    refresh: true,
  })

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: 'Refresh',
      onClick: refresh,
      enable: state.refresh,
      children: <RefreshIcon></RefreshIcon>,
    },
    {
      tooltip: 'Add',
      onClick: () => operationHandler.showAddDialog(displayItems()),
      enable: state.add,
      children: <OperationAddIcon></OperationAddIcon>,
    },
    {
      tooltip: 'Update',
      enable: state.update,
      children: <OperationUpdateIcon></OperationUpdateIcon>,
      onClick: () => operationHandler.showUpdateDialog(displayItems()),
    },
    {
      tooltip: 'Revert',
      enable: state.revert,
      children: <OperationRevertIcon></OperationRevertIcon>,
      onClick: () => operationHandler.showRevertDialog(displayItems()),
    },
    {
      tooltip: 'Diff',
      enable: state.diff,
      children: <OperationDiffIcon></OperationDiffIcon>,
      onClick: () => {
        if (!state.diff) {
          return
        }
        operationHandler.showDifferenceDialog(
          workingCopy.path,
          workspace.path,
          selection.get()[0].path,
        )
      },
    },
    {
      tooltip: 'Patch',
      enable: state.patch,
      children: <OperationPatchIcon></OperationPatchIcon>,
    },
    {
      tooltip: 'Lock',
      enable: state.lock,
      children: <OperationLockIcon />,
      onClick: () => operationHandler.showLockDialog(displayItems()),
    },
    {
      tooltip: 'Unlock',
      enable: state.unlock,
      children: <OperationUnlockIcon />,
      onClick: () => operationHandler.showUnlockDialog(displayItems()),
    },
    {
      tooltip: 'Commit',
      enable: state.commit,
      children: <OperationCommitIcon />,
      onClick: () => operationHandler.showCommitDialog(displayItems(), workingCopy.path),
    },
    {
      tooltip: 'Delete',
      enable: state.delete,
      children: <OperationDeleteIcon />,
      onClick: () => operationHandler.showDeleteDialog(displayItems()),
    },
    {
      tooltip: 'Mkdir',
      enable: state.mkdir,
      children: <OperationMkdirIcon />,
      onClick: () => operationHandler.showMkdirDialog(selection.get()[0].path),
    },
    {
      sync: false,
      tooltip: 'Info',
      enable: state.info,
      children: <OperationInfoIcon />,
      onClick: async () => {
        await Subversion.callOnce({
          factory: subversion,
          async call(context) {
            const options: InfoOptions = {
              path: selection.get()[0].path,
              pegRevision: 'unspecified',
              revision: 'unspecified',
              depth: 'empty',
              fetchExcluded: true,
              fetchActualOnly: true,
              includeExternals: true,
              changelists: null,
            }
            const result = await context.info(options)
            const entries = Object.entries(result.entries)
            if (entries.length === 1) {
              operationHandler.showInfoEntryDialog(entries[0][1])
            }
          },
        })
      },
    },
    {
      tooltip: 'Switch',
      enable: state.switch,
      children: <OperationSwitchIcon />,
      onClick: () => operationHandler.showSwitchDialog(selection.get()[0].path),
    },
    {
      tooltip: 'Merge',
      enable: state.merge,
      children: <OperationMergeIcon />,
      onClick: () =>
        operationHandler.showMergeDialog(selection.get()[0].path, {
          peg: { source: '', pegRevision: 'working', rangesToMerge: null },
        }),
    },
    {
      tooltip: 'Copy',
      enable: state.copy,
      children: <OperationCopyIcon />,
      onClick: () => {},
    },
    {
      tooltip: 'Move',
      enable: state.move,
      children: <OperationMoveIcon />,
      onClick: () => {},
    },
    {
      tooltip: 'Export',
      enable: state.export,
      children: <OperationExportIcon />,
      onClick: () => operationHandler.showExportDialog(selection.get()[0].path),
    },
  ]
  const bar = (
    <div className={cx(flex, items_center, !visible && hidden)}>
      <OperationBar
        className={cx(gap_x_1, flex_1, flex_wrap)}
        size={25}
        icons={icons}
      ></OperationBar>
    </div>
  )

  const onClick = (index: number, event: React.MouseEvent<HTMLDivElement, MouseEvent>) => {
    const call = (previous: ItemSelection<StatusEntry>) => {
      // Shift：选择连续范围
      if (event.shiftKey) {
        return previous.selectRange(index)
      }

      // Ctrl / macOS Command：切换当前项
      if (event.ctrlKey || event.metaKey) {
        return previous.selectToggle(index)
      }

      // 普通点击：只选择当前项
      return previous.select(index)
    }
    setSelection(call(selection))
  }

  // absolute inset-0 overflow-y-auto py-[2px]
  //
  const ignoreItems: ContextMenuItemModel[] = []

  let extension = null
  if (state.ignore) {
    extension = repoPath.getExtension(selection.get()[0].path)
  }

  const addIgnoreLine = async (path: string, pattern: string, recursively: boolean) => {
    const parent = repoPath.getParent(path)
    if (parent === null) {
      return
    }
    path = parent
    await Subversion.callOnce({
      factory: subversion,
      call: async (context: Subversion) => {
        const name: NodePropertyName = recursively ? 'svn:global-ignores' : 'svn:ignore'
        const options: PropertyGetOptions = {
          propertyName: name,
          target: path,
          pegRevision: 'unspecified',
          revision: 'unspecified',
          depth: 'empty',
          inherited: false,
          actualRevision: false,
          changelists: null,
        }
        const result = await context.propertyGet(options)
        const entries = new Map(Object.entries(result.properties))
        let value = entries.get(path)
        if (value) {
          value = [...value.split('\n').filter((i) => i !== ''), pattern].join('\n')
        } else {
          value = pattern
        }
        {
          const options: PropertySetOptions = {
            local: {
              name,
              value,
              targets: [path],
              depth: 'empty',
              skipChecks: false,
              changelists: null,
            },
          }
          await context.propertySet(options)
        }
      },
    })
    await refresh()
  }

  if (extension) {
    ignoreItems.push({
      item: {
        content: `Ignore *.${extension}`,
        onSelect: () => {
          if (selection.get().length === 0) {
            return
          }
          addIgnoreLine(selection.get()[0].path, `*.${extension}`, false)
        },
      },
    })
    ignoreItems.push({
      item: {
        content: `Ignore *.${extension} recursively`,
        onSelect: () => {
          if (selection.get().length === 0) {
            return
          }
          addIgnoreLine(selection.get()[0].path, `*.${extension}`, true)
        },
      },
    })
  }

  if (state.ignore) {
    const fileName = repoPath.getFileName(selection.get()[0].path)

    if (fileName) {
      ignoreItems.push({
        item: {
          content: `Ignore ${fileName}`,
          onSelect: () => {
            if (selection.get().length === 0) {
              return
            }
            addIgnoreLine(selection.get()[0].path, fileName, false)
          },
        },
      })
      ignoreItems.push({
        item: {
          content: `Ignore ${fileName} recursively`,
          onSelect: () => {
            if (selection.get().length === 0) {
              return
            }
            addIgnoreLine(selection.get()[0].path, fileName, true)
          },
        },
      })
    }
  }

  const menu: ContextMenuItemModel[] = [
    {
      item: {
        content: 'Add',
        disabled: !state.add,
        onSelect: () => {
          operationHandler.showAddDialog(displayItems())
        },
      },
    },
    {
      item: {
        content: 'Update',
        disabled: !state.update,
        onSelect: () => {
          operationHandler.showUpdateDialog(displayItems())
        },
      },
    },
    {
      subitem: {
        content: 'Ignore',
        disabled:
          !state.ignore &&
          selection.get().length !== 0 &&
          selection.get()[0].path !== workspace.path,
        items: ignoreItems,
      },
    },
    // {
    //   identity: 'Add',
    //   children: 'Add',
    //   disable: !state.add,
    //   isSeparator: false,
    //   onSelect: () => {},
    // },
    // {
    //   identity: 'Update',
    //   children: 'Update',
    //   disable: !state.update,
    //   isSeparator: false,
    //   onSelect: () => {
    //     showUpdateDialog()
    //   },
    // },
  ]

  const onContextMenu = useMemoizedFn((index: number) => {
    if (selection.isSelectedIndex(index)) {
    } else {
      setSelection((selection) => selection.select(index))
    }
  })

  return (
    <div className={cx(flex_1, flex, min_w_0, !visible && hidden)}>
      <VirtualList
        className={cx(flex_1)}
        count={entries.length}
        itemHeight={30}
        getItemKey={(index) => entries[index].path}
        itemRender={(virtualRow) => {
          const index = virtualRow.index
          const entry = entries[index]
          const model = fromStatusEntry(entry, false, workingCopy.path)
          return (
            <ContextMenu menu={menu}>
              <WorkingCopyItem
                onClick={(event) => {
                  onClick(index, event)
                }}
                onContextMenu={() => {
                  onContextMenu(index)
                }}
                {...model}
                showRelativeDirectory={true}
                className={cx(
                  list_item,
                  overflow_hidden,
                  whitespace_nowrap,
                  selection.isSelectedIndex(index) && list_item_selected,
                )}
              />
            </ContextMenu>
          )
        }}
      />
      {workingCopy.changesViewOperationContainer === null ? (
        <></>
      ) : (
        createPortal(bar, workingCopy.changesViewOperationContainer)
      )}
    </div>
  )
}
