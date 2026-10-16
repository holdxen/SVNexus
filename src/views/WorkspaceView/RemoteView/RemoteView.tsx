'use no memo'

import { IconTreeTriangleDown } from '@douyinfe/semi-icons'
import { Spin, Toast } from '@douyinfe/semi-ui'
import {
  asyncDataLoaderFeature,
  buildProxiedInstance,
  expandAllFeature,
  FeatureImplementation,
  hotkeysCoreFeature,
  selectionFeature,
} from '@headless-tree/core'
import { useTree } from '@headless-tree/react'
import OperationCheckoutIcon from '@icons/OperationCheckout.svg?react'
import OperationDeleteIcon from '@icons/OperationDelete.svg?react'
import OperationExportIcon from '@icons/OperationExport.svg?react'
import OperationInfoIcon from '@icons/OperationInfo.svg?react'
import OperationLockIcon from '@icons/OperationLock.svg?react'
import OperationMkdirIcon from '@icons/OperationMkdir.svg?react'
import OperationUnlockIcon from '@icons/OperationUnlock.svg?react'
import RefreshIcon from '@icons/Refresh.svg?react'
import StatusLockedIcon from '@icons/StatusLocked.svg?react'
import { css, cx } from '@linaria/core'
import { useVirtualizer } from '@tanstack/react-virtual'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { filesize } from 'filesize'
import { ComponentType, useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { Group, Panel, Separator } from 'react-resizable-panels'
import * as uuid from 'uuid'

import { CatOptions } from '@/bindings/CatOptions'
import { InfoEntry } from '@/bindings/InfoEntry'
import { InfoOptions } from '@/bindings/InfoOptions'
import { ListEntry } from '@/bindings/ListEntry'
import { ListOptions } from '@/bindings/ListOptions'
import { ContextMenu, ContextMenuItemModel } from '@/components/ContextMenu'
import LoadingLayer from '@/components/LoadingLayer'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import StrongEditor, { StrongEditorRef } from '@/components/StrongEditor'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { base64Encode } from '@/context/Functions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { t as translate, useT } from '@/i18n'
import { useModal } from '@/lib/multi-modal'
import {
  block_minimizable,
  flex,
  flex_1,
  flex_col,
  flex_wrap,
  gap_x_1,
  hidden,
  items_center,
  min_h_0,
  min_w_0,
  minimizable,
  overflow_hidden,
  overflow_y_auto,
  p_1,
  select_text,
  visibility_hidden,
  whitespace_nowrap,
} from '@/styles/Classes'
import Logger from '@/utils/Logger'
import { repoPath } from '@/utils/Path'
import { trimStartSubstring } from '@/utils/String'
import { combineUrl } from '@/utils/Url'
import { NiceCheckoutDialog } from '@/views/dialogs/CheckoutDialog'
import { NiceCopyDialog } from '@/views/dialogs/CopyDialog'
import { NiceCreateBranchTagDialog } from '@/views/dialogs/CreateBranchTagDialog'
import { NiceDeleteDialog } from '@/views/dialogs/DeleteDialog'
import { NiceExportDialog } from '@/views/dialogs/ExportDialog'
import { NiceInfoDialog } from '@/views/dialogs/InfoDialog'
import { NiceLockDialog } from '@/views/dialogs/LockDialog'
import { NiceMkdirDialog } from '@/views/dialogs/MkdirDialog'
import { NiceMoveDialog } from '@/views/dialogs/MoveDialog'
import { NiceMoveRenameDialog } from '@/views/dialogs/MoveRenameDialog'
import { NiceUnlockDialog } from '@/views/dialogs/UnlockDialog'

import { canPatse, useCopyMoveItems } from '../CopyMoveContext'
import { WorkingCopyPathItemModel } from '../WorkingCopyItem'
import { useWorkingCopyContext } from '../WorkingCopyView'

export interface RemoteViewProps {
  className?: string
  visible: boolean
}

interface RemoteSnapshotViewProps {
  root: string
  location: string
  pegRevision: number
  revision: number
  className?: string
  onSelectionChange: (items: RemoteSelection[]) => void
  operationMenu: ContextMenuItemModel[]
}

interface RemoteSelection {
  id: string
  entry: ListEntry
}

interface RemoteTarget {
  url: string
  root: string
  location: string
  revision: number
}

const treeNode = css`
  display: flex;
  align-items: center;
  cursor: pointer;
  box-sizing: border-box;
  color: var(--semi-color-text-0);
  transition: background-color 100ms ease;
  border: none;
  width: 100%;
  text-align: left;
  background: transparent;
  outline: none;

  &:hover {
    background-color: var(---svnexus-list-item-hover-background);
  }

  &:active {
    background-color: var(---svnexus-list-item-pressed-background);
  }

  &.${hidden} {
    display: none;
  }
`

const treeNodeSelected = css`
  background-color: var(---svnexus-list-item-selected-background);

  &:hover,
  &:active {
    background-color: var(---svnexus-list-item-selected-hover-background);
  }
`

const expandIcon = css`
  display: flex;
  flex-shrink: 0;
  color: var(--semi-color-text-2);
  transition: transform 100ms ease;
`

const expandIconCollapsed = css`
  transform: rotate(270deg);
`

const spin = css`
  .semi-spin-wrapper {
    display: flex;
    align-items: center;
  }
`

const svg = css`
  svg {
    width: 18px;
    height: 18px;
  }
`

function infoRow(key: string, value: string | null | undefined) {
  return {
    key: key,
    value: <div className={cx(select_text)}>{value}</div>,
    hidden: value === null || value === undefined,
  }
}

export function RemoteView(props: RemoteViewProps) {
  const t = useT()
  const workingCopy = useWorkingCopyContext()
  const subversion = useSubversion()
  const modal = useModal()
  const [target, setTarget] = useState<RemoteTarget | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [reload, setReload] = useState(0)
  const [selection, setSelection] = useState<RemoteSelection[]>([])
  const [snapshotKey, setSnapshotKey] = useState(0)
  const copyMove = useCopyMoveItems()

  const load = useCallback(async () => {
    setIsLoading(true)
    const result = await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        const options: InfoOptions = {
          path: workingCopy.path,
          pegRevision: 'unspecified',
          revision: 'unspecified',
          depth: 'empty',
          fetchExcluded: false,
          fetchActualOnly: false,
          includeExternals: false,
          changelists: null,
        }
        const entries = Object.entries((await context.info(options)).entries)
        if (entries.length !== 1) {
          throw new Error(`expected exactly one entry, got ${entries.length}`)
        }
        const entry = entries[0][1]
        if (entry.url === null || entry.repositoryRootUrl === null) {
          throw new Error(
            translate('changes.remote.notWorkingCopyOfRepository', { path: workingCopy.path }),
          )
        }
        const revision = await context.raGetLatestRevisionNumber(entry.url)
        return {
          url: entry.url,
          root: entry.repositoryRootUrl,
          location: trimStartSubstring(entry.url, entry.repositoryRootUrl),
          revision,
        }
      },
    })
    if (result !== null) {
      setTarget(result)
    }
    setIsLoading(false)
  }, [subversion, workingCopy.path])

  useEffect(() => {
    load()
  }, [load, reload])

  // lock/unlock 不产生新版本号，只靠 revision 变化不足以触发重载，所以额外用 snapshotKey 强制重挂载
  const refresh = async () => {
    setSelection([])
    await load()
    setSnapshotKey((value) => value + 1)
  }

  const run = async <P extends Record<string, unknown>>(component: ComponentType<P>, args: P) => {
    if (await modal.show(component, args).as<boolean>()) {
      await refresh()
    }
  }

  const displayItems = (): WorkingCopyPathItemModel[] => {
    if (target === null) {
      return []
    }
    return selection.map(({ id, entry }) => ({
      path: id,
      kind: entry.kind ?? 'unknown',
      textToolTip: id,
      fileName: repoPath.getFileName(id) ?? '/',
      relativeDirectory: trimStartSubstring(repoPath.getParent(id) ?? '', target.root),
      isLocked: entry.lock !== null,
      isLoading: false,
      status: 'normal',
      changelist: null,
      switched: false,
      copied: false,
    }))
  }

  const showInfoDialog = async () => {
    if (target === null) {
      return
    }
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const options: InfoOptions = {
          path: selection[0].id,
          pegRevision: { number: target.revision },
          revision: { number: target.revision },
          depth: 'empty',
          fetchExcluded: false,
          fetchActualOnly: false,
          includeExternals: false,
          changelists: null,
        }
        const entries = Object.entries((await context.info(options)).entries)
        if (entries.length !== 1) {
          throw new Error(`expected exactly one entry, got ${entries.length}`)
        }
        const info: InfoEntry = entries[0][1]
        const sizeText = info.size === null ? null : filesize(info.size)
        modal.show(NiceInfoDialog, {
          data: [
            infoRow('Url:', info.url),
            infoRow('Kind:', info.kind),
            infoRow('Size:', sizeText),
            infoRow('Revision:', info.revision?.toString() ?? null),
            infoRow('Last changed rev:', info.lastChangedRevision?.toString() ?? null),
            infoRow('Last changed author:', info.lastChangedAuthor),
            infoRow('Last changed date:', new Date(info.lastChangedDate).toString()),
            infoRow('Repository UUID:', info.repositoryUuid),
            infoRow('Repository root URL:', info.repositoryRootUrl),
            infoRow('Lock owner:', info.lock?.owner ?? null),
            infoRow(
              'Lock create date:',
              info.lock === null ? null : new Date(info.lock.creationDate).toString(),
            ),
          ],
        })
      },
    })
  }

  const selectedEntries = selection.map((e) => e.entry)
  const state = {
    refresh: !isLoading,
    delete: target !== null && selection.length > 0,
    mkdir: target !== null && selection.length === 1 && selectedEntries[0]?.kind === 'directory',
    lock:
      target !== null &&
      selection.length > 0 &&
      selectedEntries.every((e) => e.kind === 'file' && e.lock === null),
    unlock:
      target !== null &&
      selection.length > 0 &&
      selectedEntries.every((e) => e.kind === 'file' && e.lock !== null),
    export: target !== null && selection.length === 1,
    checkout: target !== null && selection.length === 1,
    copy: target !== null && selection.length === 1,
    move: target !== null && selection.length === 1,
    rename: target !== null && selection.length === 1,
    info: target !== null && selection.length === 1,
    paste: selection.length === 1 && selection[0].entry.kind === 'directory',
  }

  const doRefresh = async () => {
    setReload((value) => value + 1)
  }
  const doMkdir = () => run(NiceMkdirDialog, { parent: selection[0].id, needCommitMessage: true })
  const doDelete = () => run(NiceDeleteDialog, { items: displayItems(), needCommitMessage: true })
  const doLock = () => run(NiceLockDialog, { items: displayItems() })
  const doUnlock = () => run(NiceUnlockDialog, { items: displayItems() })
  const doExport = () => {
    modal.show(NiceExportDialog, {
      defaultPath: selection[0].id,
      defaultRevision: 'head',
    })
  }
  const doCheckout = () => {
    modal.show(NiceCheckoutDialog, {
      defaultUrl: selection[0].id,
    })
  }
  const doCopy = () => {
    if (target === null) {
      return
    }
    return run(NiceCreateBranchTagDialog, {
      source: {
        path: selection[0].id,
        pegRevision: { number: target.revision },
        revision: { number: target.revision },
      },
    })
  }

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: t('shared.action.refresh'),
      enable: state.refresh,
      children: <RefreshIcon></RefreshIcon>,
      onClick: doRefresh,
    },
    {
      tooltip: t('shared.action.mkdir'),
      enable: state.mkdir,
      children: <OperationMkdirIcon></OperationMkdirIcon>,
      onClick: doMkdir,
    },
    {
      tooltip: t('shared.action.delete'),
      enable: state.delete,
      children: <OperationDeleteIcon></OperationDeleteIcon>,
      onClick: doDelete,
    },
    {
      tooltip: t('shared.action.lock'),
      enable: state.lock,
      children: <OperationLockIcon></OperationLockIcon>,
      onClick: doLock,
    },
    {
      tooltip: t('shared.action.unlock'),
      enable: state.unlock,
      children: <OperationUnlockIcon></OperationUnlockIcon>,
      onClick: doUnlock,
    },
    {
      tooltip: t('shared.action.export'),
      enable: state.export,
      children: <OperationExportIcon></OperationExportIcon>,
      onClick: doExport,
    },
    {
      tooltip: t('shared.action.checkout'),
      enable: state.checkout,
      children: <OperationCheckoutIcon></OperationCheckoutIcon>,
      onClick: doCheckout,
    },
    {
      sync: false,
      tooltip: t('shared.action.info'),
      enable: state.info,
      children: <OperationInfoIcon></OperationInfoIcon>,
      onClick: showInfoDialog,
    },
  ]

  const operationMenu: ContextMenuItemModel[] = [
    {
      item: {
        content: t('shared.action.mkdir'),
        disabled: !state.mkdir,
        onSelect: doMkdir,
      },
    },
    {
      item: {
        content: t('shared.action.delete'),
        disabled: !state.delete,
        onSelect: doDelete,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.lock'),
        disabled: !state.lock,
        onSelect: doLock,
      },
    },
    {
      item: {
        content: t('shared.action.unlock'),
        disabled: !state.unlock,
        onSelect: doUnlock,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.export'),
        disabled: !state.export,
        onSelect: doExport,
      },
    },
    {
      item: {
        content: t('shared.action.checkout'),
        disabled: !state.checkout,
        onSelect: doCheckout,
      },
    },
    {
      item: {
        content: t('shared.action.copy'),
        disabled: !state.copy,
        onSelect: () => {
          copyMove.setSource({
            copy: selection.map((e) => ({
              path: e.id,
              pegRevision: 'working',
              revision: 'working',
              nodeKind: e.entry.kind ?? undefined,
            })),
          })
        },
      },
    },
    {
      item: {
        content: t('shared.action.cut'),
        disabled: !state.move,
        onSelect: () => {
          copyMove.setSource({
            isLocal: true,
            move: selection.map((e) => ({
              path: e.id,
              nodeKind: e.entry.kind ?? undefined,
            })),
          })
        },
      },
    },
    {
      item: {
        content: t('shared.action.rename'),
        disabled: !state.rename,
        onSelect: async () => {
          const result = await modal
            .show(NiceMoveRenameDialog, {
              path: selection[0].id,
              needCommitMessage: true,
            })
            .as<boolean>()
          if (result) {
            await refresh()
          }
        },
      },
    },
    {
      item: {
        content: t('shared.action.paste'),
        disabled:
          !state.paste ||
          !canPatse(copyMove.source) ||
          (copyMove.source !== undefined &&
            'move' in copyMove.source &&
            copyMove.source.isLocal === true),
        onSelect: async () => {
          if (copyMove.source !== undefined) {
            if ('copy' in copyMove.source) {
              const result = await modal
                .show(NiceCopyDialog, {
                  needCommitMessage: true,
                  sources: copyMove.source.copy,
                  defaultDestination: selection[0].id,
                })
                .as<boolean>()
              if (result) {
                await refresh()
                copyMove.setSource(undefined)
              }
            } else if ('move' in copyMove.source) {
              const result = await modal
                .show(NiceMoveDialog, {
                  sources: copyMove.source.move,
                  needCommitMessage: true,
                  defaultDestination: selection[0].id,
                })
                .as<boolean>()
              if (result) {
                await refresh()
                copyMove.setSource(undefined)
              }
            }
          }
        },
      },
    },
    {
      item: {
        content: t('shared.action.createBranchTag'),
        disabled:
          !state.copy ||
          selectedEntries.length !== 1 ||
          (selectedEntries[0].absolutePath === '/' && selectedEntries[0].path === ''),
        onSelect: doCopy,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.info'),
        disabled: !state.info,
        onSelect: showInfoDialog,
      },
    },
    { separator: {} },
    {
      item: {
        content: t('shared.action.refresh'),
        disabled: !state.refresh,
        onSelect: doRefresh,
      },
    },
  ]

  // const bar = (
  //   <div className={cx(flex, items_center, gap_x_1, overflow_hidden, operationBarRow)}>
  //     <RemoteOperationBar className={cx(gap_x_1)} size={25} icons={icons}></RemoteOperationBar>
  //     {target !== null && (
  //       <span
  //         className={cx(text_12px, whitespace_nowrap, overflow_hidden, min_w_0)}
  //         style={{ color: 'var(--semi-color-text-2)' }}
  //         title={`${target.url} @ r${target.revision}`}
  //       >
  //         {target.url} @ r{target.revision}
  //       </span>
  //     )}
  //   </div>
  // )

  const bar = (
    <div className={cx(flex, items_center, !props.visible && hidden)}>
      <OperationBar
        className={cx(gap_x_1, flex_1, flex_wrap)}
        size={25}
        icons={icons}
      ></OperationBar>
    </div>
  )

  return (
    <div className={cx(props.className, flex, flex_col, minimizable, overflow_hidden)}>
      {workingCopy.remoteViewOperationContainer !== null &&
        createPortal(bar, workingCopy.remoteViewOperationContainer)}
      <LoadingLayer className={cx(flex_1)} state={isLoading ? 'loading' : 'none'}>
        {target !== null && (
          <RemoteSnapshotView
            key={`${target.url}_${target.revision}_${snapshotKey}`}
            className={cx(flex_1, block_minimizable)}
            root={target.root}
            location={target.location}
            pegRevision={target.revision}
            revision={target.revision}
            onSelectionChange={setSelection}
            operationMenu={operationMenu}
          ></RemoteSnapshotView>
        )}
      </LoadingLayer>
    </div>
  )
}

function RemoteSnapshotView(props: RemoteSnapshotViewProps) {
  const t = useT()
  const subversion = useSubversion()
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const editor = useRef<StrongEditorRef | null>(null)
  const [currentEditor, setCurrentEditor] = useState<string>()
  const [isLoading, setIsLoading] = useState(true)

  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizerRef = useRef<any>(null)
  const resizeObserverRef = useRef<ResizeObserver | null>(null)
  const [isReady, setIsReady] = useState(false)

  const doubleClickBehavior: FeatureImplementation = {
    itemInstance: {
      getProps: ({ item, prev }) => ({
        ...prev?.(),
        // 双击时展开/折叠
        onDoubleClick: () => {
          item.primaryAction()

          if (!item.isFolder()) return

          if (item.isExpanded()) {
            item.collapse()
          } else {
            item.expand()
          }
        },
        onClick: () => {
          item.setFocused()
        },
      }),
    },
  }

  const root = ''

  const url = useMemo(() => {
    return combineUrl(props.root, props.location)
  }, [props.root, props.location])

  const tree = useTree<ListEntry>({
    instanceBuilder: buildProxiedInstance,
    scrollToItem: (item) => {
      virtualizerRef.current?.scrollToIndex(item.getItemMeta().index)
    },
    state: {
      selectedItems,
    },
    setSelectedItems,
    initialState: {
      expandedItems: [url],
      selectedItems,
    },
    getItemName: (item) => {
      let path = item.getItemData().path
      if (path === root) {
        return repoPath.getFileName(url) ?? '/'
      }
      return path
    },
    isItemFolder: (item) => {
      return item.getItemData().kind === 'directory'
    },
    dataLoader: {
      getChildrenWithData: async (itemId: string) => {
        setIsLoading(itemId === root)
        const result = await Subversion.callOnce({
          factory: subversion,
          call: async (context) => {
            const isRoot = itemId === root
            const options: ListOptions = {
              path: isRoot ? url : itemId,
              pegRevision: { number: props.pegRevision },
              revision: { number: props.revision },
              patterns: null,
              depth: isRoot ? 'empty' : 'immediates',
              direntCreatedRevision: false,
              direntHasProperties: false,
              direntKind: true,
              direntLastAuthor: false,
              direntSize: true,
              direntTime: false,
              fetchLocks: true,
              includeExternals: false,
            }

            const fileEntries: ListEntry[] = []
            const dirEntries: ListEntry[] = []

            let entries = (await context.list(options)).entries

            for (let i of entries) {
              if (i.kind === 'directory') {
                dirEntries.push(i)
              } else {
                fileEntries.push(i)
              }
            }

            dirEntries.sort((a, b) => a.path.localeCompare(b.path))
            fileEntries.sort((a, b) => a.path.localeCompare(b.path))

            entries = [...dirEntries, ...fileEntries]

            if (isRoot) {
              if (entries.length !== 1) {
                throw new Error('expected exactly one entry, got ' + entries.length)
              }
              const entry = entries[0]
              return [{ id: url, data: { ...entry, name: '/' } }]
            } else {
              const result: { id: string; data: ListEntry }[] = []
              for (let i of entries) {
                if (i.path === '') {
                  continue
                }
                result.push({ id: combineUrl(itemId, i.path), data: i })
              }
              return result
            }
          },
        })
        setIsLoading(false)
        return result ?? []
      },
      getItem: async (itemId: string) => {
        const options: ListOptions = {
          path: itemId === root ? url : itemId,
          pegRevision: { number: props.pegRevision },
          revision: { number: props.revision },
          patterns: null,
          depth: 'empty',
          direntCreatedRevision: false,
          direntHasProperties: false,
          direntKind: true,
          direntLastAuthor: false,
          direntSize: true,
          direntTime: false,
          fetchLocks: true,
          includeExternals: false,
        }
        const result = await Subversion.callOnce({
          factory: subversion,
          call: async (context) => {
            const result = await context.list(options)
            if (result.entries.length !== 1) {
              throw new Error('expected exactly one entry')
            }
            return result.entries[0]
          },
        })
        if (result === null) {
          throw new Error(`Failed to get itemId:${itemId}`)
        }
        return result
      },
    },
    rootItemId: root,
    features: [
      doubleClickBehavior,
      asyncDataLoaderFeature,
      selectionFeature,
      hotkeysCoreFeature,
      expandAllFeature,
    ],
  })

  useEffect(() => {
    props.onSelectionChange(
      tree.getSelectedItems().map((e) => ({
        id: e.getItemMeta().itemId,
        entry: e.getItemData(),
      })),
    )
  }, [selectedItems])

  useEffect(() => {
    if (selectedItems.length === 0) {
      setCurrentEditor(undefined)
    } else if (selectedItems.length === 1) {
      const item = tree.getItemInstance(selectedItems[0])
      if (item.getItemData().kind === 'file') {
        setCurrentEditor(selectedItems[0])
        const path = item.getItemMeta().itemId
        const options: CatOptions = {
          path,
          pegRevision: { number: props.revision },
          revision: { number: props.revision },
          expandKeywords: false,
          getProperties: false,
        }

        editor.current?.add(path, async () => {
          const result = await Subversion.callOnce({
            factory: subversion,
            call: async (context) => {
              const result = await context.cat(options)
              const decoder = new TextDecoder('utf-8', { fatal: true })
              return decoder.decode(result.content)
            },
          })

          return result ?? ''
        })
      }
    }
  }, [selectedItems])

  const treeItems = tree.getItems()

  const virtualizer = useVirtualizer({
    count: treeItems.length,
    getScrollElement: () => {
      if (!isReady) return null
      return scrollRef.current
    },
    estimateSize: () => 30,
    overscan: 5,
    getItemKey: (index) => treeItems[index].getId(),
  })

  virtualizerRef.current = virtualizer

  useEffect(() => {
    setIsReady(true)

    const observer = new ResizeObserver((entries) => {
      for (const entry of entries) {
        if (entry.contentRect.width > 0 && entry.contentRect.height > 0) {
          virtualizerRef.current?.measure()
        }
      }
    })
    if (scrollRef.current) {
      observer.observe(scrollRef.current)
    }
    resizeObserverRef.current = observer

    return () => {
      resizeObserverRef.current?.disconnect()
    }
  }, [])

  return (
    <div className={cx(props.className, min_w_0)}>
      <Group orientation="horizontal" className={cx(min_w_0)}>
        <Panel className={cx(flex)} defaultSize={'30%'}>
          <Spin
            spinning={isLoading}
            wrapperClassName={cx(flex_1, flex, min_w_0, min_h_0)}
            childStyle={{ display: 'flex', minWidth: 0, minHeight: 0, flex: 1 }}
          >
            <div className={cx(flex_1, min_w_0, min_h_0, overflow_y_auto)} ref={scrollRef}>
              <div
                {...tree.getContainerProps()}
                style={{
                  height: virtualizer.getTotalSize(),
                  position: 'relative',
                  width: '100%',
                }}
              >
                {virtualizer.getVirtualItems().map((virtualItem) => {
                  const item = treeItems[virtualItem.index]
                  const itemProps = item.getProps()
                  const level = item.getItemMeta().level
                  const isFolder = item.isFolder()
                  const isExpanded = item.isExpanded()
                  const isSelected = item.isSelected()
                  const isLoading = item.isLoading()

                  const menu: ContextMenuItemModel[] = [...props.operationMenu]
                  if (item.getItemData().kind === 'file') {
                    menu.push({ separator: {} })
                    menu.push({
                      item: {
                        content: t('shared.action.save'),
                        onSelect: async () => {
                          const result = await save({
                            title: t('workspace.snapshot.saveFileTitle'),
                            defaultPath: item.getItemData().path,
                            canCreateDirectories: true,
                          })
                          if (result) {
                            await Subversion.callOnce({
                              factory: subversion,
                              call: async (context) => {
                                const path = item.getItemMeta().itemId
                                const options: CatOptions = {
                                  path,
                                  pegRevision: { number: props.revision },
                                  revision: { number: props.revision },
                                  expandKeywords: false,
                                  getProperties: false,
                                }
                                const content = await context.cat(options)

                                await writeFile(result, content.content)
                                Toast.success({
                                  content: t('changes.snapshot.saveSuccess', { path: result }),
                                  stack: true,
                                })
                              },
                              onError: (e) => {
                                Toast.error({
                                  content: t('changes.snapshot.saveFailed', {
                                    path: result,
                                    error: String(e),
                                  }),
                                  stack: true,
                                })
                              },
                            })
                          }
                        },
                        disabled: item.getItemData().kind !== 'file',
                      },
                    })
                    menu.push({
                      item: {
                        content: t('shared.action.fileHistory'),
                        onSelect: async () => {
                          Logger.info('open file history: ', props.revision)
                          try {
                            const id = uuid.v4()
                            const itemId = item.getItemMeta().itemId
                            const url = `/FileHistoryView/${await base64Encode(new TextEncoder().encode(itemId), false)}/${props.revision}`
                            const window = new WebviewWindow(id, {
                              url,
                              title: itemId,
                              width: 800,
                              height: 600,
                            })
                            await window.show()
                          } catch (e) {
                            console.error('Failed to create window', e)
                          }
                        },
                        disabled: false,
                      },
                    })
                  }

                  return (
                    <ContextMenu
                      menu={menu}
                      {...itemProps}
                      onContextMenu={() => {
                        if (!isSelected) {
                          const id = item.getItemMeta().itemId
                          setSelectedItems([id])
                          props.onSelectionChange([{ id, entry: item.getItemData() }])
                        }
                      }}
                      key={virtualItem.key}
                      data-index={virtualItem.index}
                      ref={itemProps.ref}
                      className={cx(
                        treeNode,
                        isSelected && treeNodeSelected,
                        p_1,
                        flex,
                        gap_x_1,
                        items_center,
                        whitespace_nowrap,
                        overflow_hidden,
                      )}
                      style={{
                        position: 'absolute',
                        top: 0,
                        left: 0,
                        width: '100%',
                        transform: `translateY(${virtualItem.start}px)`,
                        paddingLeft: level * 20 + 8,
                        borderRadius: 'var(--semi-border-radius-medium, 6px)',
                      }}
                    >
                      <Spin
                        size="small"
                        wrapperClassName={cx(flex, items_center, spin)}
                        spinning={isLoading}
                      >
                        <span
                          className={cx(
                            expandIcon,
                            (!isFolder || isLoading) && visibility_hidden,
                            isFolder && !isExpanded && expandIconCollapsed,
                          )}
                        >
                          <IconTreeTriangleDown
                            size="default"
                            onClick={() => {
                              const call = item.isExpanded() ? item.collapse : item.expand
                              call()
                            }}
                          />
                        </span>
                      </Spin>
                      <div className={cx(svg, flex, flex_1, items_center, min_w_0)}>
                        <FileKindIcon kind={item.getItemData().kind ?? 'unknown'}></FileKindIcon>
                        <span className={cx(flex_1, min_w_0, overflow_hidden)}>
                          {item.getItemName()}
                        </span>
                        <StatusLockedIcon
                          className={cx(item.getItemData().lock === null && hidden)}
                        ></StatusLockedIcon>
                      </div>
                    </ContextMenu>
                  )
                })}
              </div>
            </div>
          </Spin>
        </Panel>
        <Separator style={{ width: 4 }}></Separator>
        <Panel className={cx(flex)}>
          <StrongEditor
            currentKey={currentEditor}
            ref={editor}
            className={cx(flex_1)}
          ></StrongEditor>
        </Panel>
      </Group>
    </div>
  )
}
