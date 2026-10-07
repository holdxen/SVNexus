import { Toast } from '@douyinfe/semi-ui'
import OperationAddIcon from '@icons/OperationAdd.svg?react'
import OperationCommitIcon from '@icons/OperationCommit.svg?react'
import OperationDeleteIcon from '@icons/OperationDelete.svg?react'
import OperationDiffIcon from '@icons/OperationDiff.svg?react'
import OperationExportIcon from '@icons/OperationExport.svg?react'
import OperationInfoIcon from '@icons/OperationInfo.svg?react'
import OperationLockIcon from '@icons/OperationLock.svg?react'
import OperationMergeIcon from '@icons/OperationMerge.svg?react'
import OperationMkdirIcon from '@icons/OperationMkdir.svg?react'
import OperationPatchIcon from '@icons/OperationPatch.svg?react'
import OperationRevertIcon from '@icons/OperationRevert.svg?react'
import OperationSwitchIcon from '@icons/OperationSwitch.svg?react'
import OperationUnlockIcon from '@icons/OperationUnlock.svg?react'
import OperationUpdateIcon from '@icons/OperationUpdate.svg?react'
import RefreshIcon from '@icons/Refresh.svg?react'
import { cx } from '@linaria/core'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
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
import { useT } from '@/i18n'
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
import Logger from '@/utils/Logger'
import { repoPath } from '@/utils/Path'
import itemSelection, { ItemSelection } from '@/utils/selection/immutable'

import { StatusEntry } from '../../../../bindings/StatusEntry'
import { useCopyMoveItems } from '../../CopyMoveContext'
import { defaultOperationState, OperationState } from '../../Operation'
import { fromStatusEntry, WorkingCopyItem } from '../../WorkingCopyItem'
import { useWorkingCopyContext } from '../../WorkingCopyView'
import { useWorkspaceContext } from '../../WorkspaceView'
import OperationHandler from '../OperationHandler'

export interface ChangesListViewProps {
  visible: boolean
  onSelected?: (entry: string | null) => void
  onRefresh?: () => void
  statusEntries?: StatusEntry[]
  onStatusEntriesChanged?: (entries: StatusEntry[]) => void
}

export function ChangesListView({
  visible,
  onSelected,
  onRefresh,
  statusEntries,
  onStatusEntriesChanged,
}: ChangesListViewProps) {
  const t = useT()
  const workingCopy = useWorkingCopyContext()
  const workspace = useWorkspaceContext()
  const subversion = useSubversion()
  const copyItems = useCopyMoveItems()

  // const [entries, setEntries] = useState<StatusEntry[]>([])
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
        onStatusEntriesChanged?.(result.entries)
        // setEntries(result.entries)
        setSelection(itemSelection(result.entries))
      },
      onError: (error: any) => {
        Toast.error({
          content: t('changes.list.failedToGetStatus', { error: errorHumanString(error) }),
          stack: true,
        })
      },
    })
    onRefresh?.()
  }
  useEffect(() => {
    if (statusEntries !== undefined) {
      return
    }
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
      history:
        length === 1 &&
        selectedEntries[0].nodeKind === 'file' &&
        selectedEntries[0].nodeStatus !== 'unversioned' &&
        selectedEntries[0].nodeStatus !== 'added',
      copy:
        length > 0 &&
        selectedEntries.every((e) => e.nodeStatus !== 'added' && e.nodeStatus !== 'unversioned'),
      switch:
        length === 1 &&
        selectedEntries[0].nodeStatus !== 'unversioned' &&
        selectedEntries[0].nodeStatus !== 'added',
      merge:
        length === 1 &&
        selectedEntries[0].nodeStatus !== 'unversioned' &&
        selectedEntries[0].nodeStatus !== 'added',
    }
  }

  const state = selectionChanged(selection, {
    ...defaultOperationState,
    refresh: true,
  })

  const showInfo = async () => {
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
  }

  const applyPatch = async () => {
    const file = await open({
      multiple: false,
      directory: false,
    })
    if (file === null) {
      return
    }

    operationHandler.showPatchDialog(file, selection.get()[0].path)
  }

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: t('shared.action.refresh'),
      onClick: refresh,
      enable: state.refresh,
      children: <RefreshIcon></RefreshIcon>,
    },
    {
      tooltip: t('shared.action.add'),
      onClick: () => operationHandler.showAddDialog(displayItems()),
      enable: state.add,
      children: <OperationAddIcon></OperationAddIcon>,
    },
    {
      tooltip: t('shared.action.update'),
      enable: state.update,
      children: <OperationUpdateIcon></OperationUpdateIcon>,
      onClick: () => operationHandler.showUpdateDialog(displayItems()),
    },
    {
      tooltip: t('shared.action.revert'),
      enable: state.revert,
      children: <OperationRevertIcon></OperationRevertIcon>,
      onClick: () => operationHandler.showRevertDialog(displayItems()),
    },
    {
      tooltip: t('shared.action.diff'),
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
      tooltip: t('shared.action.patch'),
      enable: state.patch,
      children: <OperationPatchIcon></OperationPatchIcon>,
      onClick: applyPatch,
    },
    {
      tooltip: t('shared.action.lock'),
      enable: state.lock,
      children: <OperationLockIcon />,
      onClick: () => operationHandler.showLockDialog(displayItems()),
    },
    {
      tooltip: t('shared.action.unlock'),
      enable: state.unlock,
      children: <OperationUnlockIcon />,
      onClick: () => operationHandler.showUnlockDialog(displayItems()),
    },
    {
      tooltip: t('shared.action.commit'),
      enable: state.commit,
      children: <OperationCommitIcon />,
      onClick: () => operationHandler.showCommitDialog(displayItems(), workingCopy.path),
    },
    {
      tooltip: t('shared.action.delete'),
      enable: state.delete,
      children: <OperationDeleteIcon />,
      onClick: () => operationHandler.showDeleteDialog(displayItems()),
    },
    {
      tooltip: t('shared.action.mkdir'),
      enable: state.mkdir,
      children: <OperationMkdirIcon />,
      onClick: () => operationHandler.showMkdirDialog(selection.get()[0].path),
    },
    {
      sync: false,
      tooltip: t('shared.action.info'),
      enable: state.info,
      children: <OperationInfoIcon />,
      onClick: showInfo,
    },
    {
      tooltip: t('shared.action.switch'),
      enable: state.switch,
      children: <OperationSwitchIcon />,
      onClick: () => operationHandler.showSwitchDialog(selection.get()[0].path),
    },
    {
      tooltip: t('shared.action.merge'),
      enable: state.merge,
      children: <OperationMergeIcon />,
      onClick: () =>
        operationHandler.showMergeDialog(selection.get()[0].path, {
          peg: { source: '', pegRevision: 'working', rangesToMerge: null },
        }),
    },
    {
      tooltip: t('shared.action.export'),
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
        content: t('changes.ignore.extension', { extension }),
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
        content: t('changes.ignore.extensionRecursively', { extension }),
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
          content: t('changes.ignore.name', { name: fileName }),
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
          content: t('changes.ignore.nameRecursively', { name: fileName }),
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
        content: t('shared.action.add'),
        disabled: !state.add,
        onSelect: () => {
          operationHandler.showAddDialog(displayItems())
        },
      },
    },
    {
      item: {
        content: t('shared.action.update'),
        disabled: !state.update,
        onSelect: () => {
          operationHandler.showUpdateDialog(displayItems())
        },
      },
    },
    {
      item: {
        content: t('shared.action.commit'),
        disabled: !state.commit,
        onSelect: () => operationHandler.showCommitDialog(displayItems(), workingCopy.path),
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.diff'),
        disabled: !state.diff,
        onSelect: () => {
          operationHandler.showDifferenceDialog(
            workingCopy.path,
            workspace.path,
            selection.get()[0].path,
          )
        },
      },
    },
    {
      item: {
        content: t('shared.action.revert'),
        disabled: !state.revert,
        onSelect: () => operationHandler.showRevertDialog(displayItems()),
      },
    },
    {
      item: {
        content: t('shared.action.patch'),
        disabled: !state.patch,
        onSelect: applyPatch,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.delete'),
        disabled: !state.delete,
        onSelect: () => operationHandler.showDeleteDialog(displayItems()),
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.lock'),
        disabled: !state.lock,
        onSelect: () => operationHandler.showLockDialog(displayItems()),
      },
    },
    {
      item: {
        content: t('shared.action.unlock'),
        disabled: !state.unlock,
        onSelect: () => operationHandler.showUnlockDialog(displayItems()),
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.mkdir'),
        disabled: !state.mkdir,
        onSelect: () => operationHandler.showMkdirDialog(selection.get()[0].path),
      },
    },
    {
      item: {
        content: t('shared.action.export'),
        disabled: !state.export,
        onSelect: () => operationHandler.showExportDialog(selection.get()[0].path),
      },
    },
    {
      item: {
        content: t('shared.action.switch'),
        disabled: !state.switch,
        onSelect: () => operationHandler.showSwitchDialog(selection.get()[0].path),
      },
    },
    {
      item: {
        content: t('shared.action.merge'),
        disabled: !state.merge,
        onSelect: () =>
          operationHandler.showMergeDialog(selection.get()[0].path, {
            peg: { source: '', pegRevision: 'working', rangesToMerge: null },
          }),
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.copy'),
        disabled: !state.copy,
        onSelect: () => {
          copyItems.setSource({
            copy: selection.get().map((e) => ({
              path: e.path,
              pegRevision: 'working',
              revision: 'working',
            })),
          })
        },
      },
    },
    {
      item: {
        content: t('shared.action.fileHistory'),
        disabled: !state.history,
        onSelect: async () => {
          Logger.info('Open file history: ', selection.get())
          operationHandler.showFileHistoryDialog(selection.get()[0])
        },
      },
    },
    {
      item: {
        content: t('shared.action.info'),
        disabled: !state.info,
        onSelect: showInfo,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.copyAbsolutePath'),
        disabled: selection.get().length !== 1,
        onSelect: () => {
          writeText(selection.get()[0].path)
        },
      },
    },
    {
      item: {
        content: t('shared.action.copyRelativePath'),
        disabled: selection.get().length !== 1,
        onSelect: () => {
          const path = repoPath.stripPrefix(selection.get()[0].path, workingCopy.path)
          if (path !== null) {
            writeText(path)
          }
        },
      },
    },
    {
      item: {
        content: t('workspace.changes.revealInFolder'),
        disabled: selection.get().length === 0,
        onSelect: async () => {
          const paths = selection
            .get()
            .map((e) => e.path)
            .filter((p) => p !== '')
          if (paths.length === 0) {
            return
          }
          try {
            await revealItemInDir(paths)
          } catch {
            // 已删除等不存在的路径无法解析，退回打开其所在文件夹
            const parents = paths
              .map((p) => repoPath.getParent(p))
              .filter((p): p is string => p !== null && p !== '')
            if (parents.length > 0) {
              await revealItemInDir(parents).catch((error) => {
                Logger.info('revealItemInDir failed: ', error)
              })
            }
          }
        },
      },
    },
    {
      subitem: {
        content: t('shared.action.ignore'),
        disabled:
          !state.ignore &&
          selection.get().length !== 0 &&
          selection.get()[0].path !== workspace.path,
        items: ignoreItems,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.refresh'),
        disabled: !state.refresh,
        onSelect: refresh,
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
        count={statusEntries?.length ?? 0}
        itemHeight={30}
        getItemKey={(index) => statusEntries?.[index].path ?? ''}
        itemRender={(virtualRow) => {
          const index = virtualRow.index
          const entry = statusEntries?.[index]
          if (entry === undefined) {
            return <></>
          }
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
      {workingCopy.changesViewOperationContainer !== null &&
        createPortal(bar, workingCopy.changesViewOperationContainer)}
    </div>
  )
}
