'use no memo'
import { IconTreeTriangleDown } from '@douyinfe/semi-icons'
import { Checkbox } from '@douyinfe/semi-ui/lib/es/checkbox'
import {
  AsyncDataLoaderDataRef,
  asyncDataLoaderFeature,
  buildProxiedInstance,
  expandAllFeature,
  FeatureImplementation,
  hotkeysCoreFeature,
  selectionFeature,
  TreeInstance,
} from '@headless-tree/core'
import { useTree } from '@headless-tree/react'
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
import { cx, css } from '@linaria/core'
import { useVirtualizer } from '@tanstack/react-virtual'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { open } from '@tauri-apps/plugin-dialog'
import { revealItemInDir } from '@tauri-apps/plugin-opener'
import { type MouseEvent, useEffect, useMemo, useRef, useState } from 'react'
import { createPortal } from 'react-dom'

import { InfoOptions } from '@/bindings/InfoOptions'
import { NodePropertyName } from '@/bindings/NodePropertyName'
import { PropertyGetOptions } from '@/bindings/PropertyGetOptions'
import { PropertySetOptions } from '@/bindings/PropertySetOptions'
import { StatusEntry } from '@/bindings/StatusEntry'
import { StatusOptions } from '@/bindings/StatusOptions'
import { ContextMenu, ContextMenuItemModel } from '@/components/ContextMenu'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import TreeCollapseIcon from '@/icons/TreeCollapse.svg?react'
import TreeExpandIcon from '@/icons/TreeExpand.svg?react'
import { useModal } from '@/lib/multi-modal'
import {
  flex,
  flex_1,
  gap_x_1,
  hidden,
  items_center,
  overflow_hidden,
  overflow_y_auto,
  visibility_hidden,
  whitespace_nowrap,
} from '@/styles/Classes'
import Logger from '@/utils/Logger'
import { localPath, repoPath } from '@/utils/Path'
import { NiceCopyDialog } from '@/views/dialogs/CopyDialog'
import { NiceMoveDialog } from '@/views/dialogs/MoveDialog'
import { NiceMoveRenameDialog } from '@/views/dialogs/MoveRenameDialog'

import { canPatse, useCopyMoveItems } from '../../CopyMoveContext'
import { defaultOperationState, OperationState } from '../../Operation'
import { fromStatusEntry, WorkingCopyItem } from '../../WorkingCopyItem'
import { useWorkingCopyContext } from '../../WorkingCopyView'
import { useWorkspaceContext } from '../../WorkspaceView'
import OperationHandler from '../OperationHandler'

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
  margin-right: 8px;
  transition: transform 100ms ease;
`

const expandIconCollapsed = css`
  transform: rotate(270deg);
`

interface ChangesTreeViewProps {
  visible: boolean
  optionBarContainer: HTMLElement | null
  onSelected?: (entry: string | null) => void
  onRefresh?: () => void
  statusEntries?: StatusEntry[]
  onStatusEntriesChanged?: (entries: StatusEntry[]) => void
}

interface TreeEntry extends StatusEntry {
  name: string
}

async function refreshTree<T>(tree: TreeInstance<T>) {
  const { rootItemId } = tree.getConfig()
  // const expandedItems = new Set(tree.getState().expandedItems);
  //
  const dataRef = tree.getDataRef<AsyncDataLoaderDataRef>()

  async function refreshChildren(itemId: string) {
    const item = tree.getItemInstance(itemId)
    if (dataRef.current.childrenIds[itemId] === undefined) {
      return
    }

    // 强制重新获取子节点（清除缓存并重新请求）
    await item.invalidateChildrenIds()
    const childrenIds = tree.retrieveChildrenIds(itemId)

    for (const childId of childrenIds) {
      await refreshChildren(childId)
    }
  }

  await refreshChildren(rootItemId)
  tree.rebuildTree()
}

function getOperationState(tree: TreeInstance<TreeEntry>, root: string): OperationState {
  const selectedEntries = tree.getSelectedItems().map((e) => e.getItemData())
  const length = selectedEntries.length
  return {
    ...defaultOperationState,
    refresh: true,
    add: length > 0,
    revert: length > 0,
    lock:
      length > 0 &&
      selectedEntries.every(
        (e) =>
          e.nodeKind !== 'directory' &&
          e.lock === null &&
          e.nodeStatus !== 'unversioned' &&
          e.nodeStatus !== 'added',
      ),
    unlock:
      length > 0 &&
      selectedEntries.every(
        (e) =>
          e.nodeKind !== 'directory' &&
          e.lock !== null &&
          e.nodeStatus !== 'unversioned' &&
          e.nodeStatus !== 'added',
      ),
    diff: length === 1 && selectedEntries[0].nodeStatus !== 'unversioned',
    patch:
      length === 1 &&
      selectedEntries[0].nodeKind == 'directory' &&
      selectedEntries[0].nodeStatus !== 'unversioned',
    delete: length > 0,
    info: length === 1 && selectedEntries[0].nodeStatus !== 'unversioned',
    commit: length > 0 && selectedEntries.every((e) => e.nodeStatus !== 'unversioned'),
    switch:
      length == 1 &&
      selectedEntries[0].nodeStatus !== 'unversioned' &&
      selectedEntries[0].nodeStatus !== 'added',
    relocate: length == 1 && selectedEntries[0].path === root,
    mkdir:
      length === 1 &&
      selectedEntries[0].nodeKind == 'directory' &&
      selectedEntries[0].nodeStatus !== 'unversioned',
    update:
      length > 0 &&
      selectedEntries.every((e) => e.nodeStatus !== 'unversioned' && e.nodeStatus !== 'added'),
    export:
      length === 1 &&
      selectedEntries[0].nodeKind === 'directory' &&
      selectedEntries[0].nodeStatus !== 'unversioned' &&
      selectedEntries[0].nodeStatus !== 'added',
    merge:
      length === 1 &&
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
    move:
      length > 0 &&
      selectedEntries.every((e) => e.nodeStatus !== 'added' && e.nodeStatus !== 'unversioned'),
    paste:
      length === 1 &&
      selectedEntries[0].nodeKind === 'directory' &&
      selectedEntries[0].nodeStatus !== 'unversioned' &&
      selectedEntries[0].nodeStatus !== 'added',
    rename:
      length === 1 &&
      selectedEntries[0].nodeStatus !== 'unversioned' &&
      selectedEntries[0].nodeStatus !== 'missing' &&
      selectedEntries[0].nodeStatus !== 'deleted' &&
      selectedEntries[0].nodeStatus !== 'added',
  }
}

export function ChangesTreeView(props: ChangesTreeViewProps) {
  const t = useT()
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const [loadingItemData, setLoadingItemData] = useState<string[]>([])
  const [loadingItemChildrens, setLoadingItemChildrens] = useState<string[]>([])

  const workingCopy = useWorkingCopyContext()
  const subversion = useSubversion()
  const copyMove = useCopyMoveItems()

  const loadingItem: TreeEntry = {
    path: '',
    name: t('workspace.changesTree.loading'),
    nodeKind: 'directory',
    localAbsolutePath: '',
    fileSize: null,
    versioned: false,
    conflicted: false,
    nodeStatus: 'normal',
    textStatus: 'normal',
    propertyStatus: 'normal',
    wcIsLocked: false,
    copied: false,
    repositoryRootUrl: null,
    repositoryUuid: null,
    repositoryRelpath: null,
    revision: null,
    lastChangedRevision: null,
    lastChangedDate: 0,
    lastChangedAuthor: null,
    switched: false,
    fileExternal: false,
    lock: null,
    changelist: null,
    depth: 'unknown',
    outOfDateKind: 'file',
    repositoryNodeStatus: 'normal',
    repositoryTextStatus: 'normal',
    repositoryPropertyStatus: 'normal',
    repositoryLock: null,
    outOfDateChangedRevision: null,
    outOfDateChangedDate: null,
    outOfDateChangedAuthor: null,
    movedFromAbsolutePath: null,
    movedToAbsolutePath: null,
  }

  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizerRef = useRef<any>(null)
  const resizeObserverRef = useRef<ResizeObserver | null>(null)
  const [isReady, setIsReady] = useState(false)

  const doubleClickBehavior: FeatureImplementation = {
    itemInstance: {
      getProps: ({ item, prev, tree, itemId }) => ({
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
        // 右键点击时先选中该项，再由 ContextMenu 打开右键菜单。
        // 与 selectionFeature 的 onClick 保持一致：shift 范围选、ctrl/cmd 切换选、普通右键单选。
        onContextMenu: (e: MouseEvent) => {
          if (e.shiftKey) {
            item.selectUpTo(e.ctrlKey || e.metaKey)
          } else if (e.ctrlKey || e.metaKey) {
            item.toggleSelect()
          } else if (!item.isSelected()) {
            // 已处于多选中时保留原选区，避免右键菜单的操作目标丢失
            tree.setSelectedItems([itemId])
          }
          item.setFocused()
          prev?.()?.onContextMenu?.(e)
        },
      }),
    },
  }

  const root = ''

  const getItem = async (itemId: string) => {
    Logger.info('on get tree item', itemId)
    if (itemId === root) {
      return loadingItem
    }
    const options: StatusOptions = {
      path: itemId,
      revision: 'working',
      depth: 'empty',
      getAll: true,
      checkOutOfDate: false,
      checkWorkingCopy: false,
      noIgnore: false,
      ignoreExternals: false,
      depthAsSticky: false,
      changelist: null,
    }
    const result = await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        const result = await context.status(options)
        if (result.entries.length !== 1) {
          throw new Error(t('changes.error.emptyWorkingCopy'))
        }
        const entry = result.entries[0]
        const name = localPath.getFileName(entry.path) ?? ''

        return {
          ...entry,
          name,
        }
      },
    })
    return result ?? loadingItem
  }

  const tree = useTree<TreeEntry>({
    instanceBuilder: buildProxiedInstance,
    createLoadingItemData: () => loadingItem,
    rootItemId: root,
    state: {
      selectedItems,
      loadingItemData,
      loadingItemChildrens,
    },
    setSelectedItems,
    setLoadingItemData,
    setLoadingItemChildrens,
    initialState: {
      expandedItems: [workingCopy.path],
      selectedItems,
    },
    scrollToItem: (item) => {
      virtualizerRef.current?.scrollToIndex(item.getItemMeta().index)
    },
    dataLoader: {
      getChildrenWithData: async (itemId: string) => {
        const options: StatusOptions =
          itemId === root
            ? {
                path: workingCopy.path,
                revision: 'working',
                depth: 'empty',
                getAll: true,
                checkOutOfDate: false,
                checkWorkingCopy: false,
                noIgnore: false,
                ignoreExternals: false,
                depthAsSticky: false,
                changelist: null,
              }
            : {
                path: itemId,
                revision: 'working',
                depth: 'immediates',
                getAll: true,
                checkOutOfDate: false,
                checkWorkingCopy: false,
                noIgnore: false,
                ignoreExternals: false,
                depthAsSticky: false,
                changelist: null,
              }
        const result = await Subversion.callOnce({
          factory: subversion,
          call: async (context) => {
            const result = await context.status(options)

            const directoryEntries: { id: string; data: TreeEntry }[] = []
            const fileEntries: { id: string; data: TreeEntry }[] = []
            for (let entry of result.entries) {
              if (entry.path === itemId) {
                continue
              }

              let name = localPath.getFileName(entry.path) ?? ''

              let item: TreeEntry = {
                ...entry,
                name,
              }
              if (item.nodeKind === 'directory') {
                directoryEntries.push({
                  data: item,
                  id: entry.path,
                })
              } else {
                fileEntries.push({
                  data: item,
                  id: entry.path,
                })
              }
            }

            directoryEntries.sort((a, b) => a.data.name.localeCompare(b.data.name))
            fileEntries.sort((a, b) => a.data.name.localeCompare(b.data.name))

            return [...directoryEntries, ...fileEntries]
          },
        })

        return result ?? []
      },
      getItem,
    },
    getItemName: (item) => {
      return item.getItemData().name
    },
    isItemFolder: (item) => {
      return item.getItemData().nodeKind === 'directory'
    },
    features: [
      doubleClickBehavior,
      asyncDataLoaderFeature,
      selectionFeature,
      hotkeysCoreFeature,
      expandAllFeature,
    ],
  })
  const workspace = useWorkspaceContext()

  const state = getOperationState(tree, workspace.path)
  const modal = useModal()

  const refresh = async () => {
    await refreshTree(tree)
    await refreshStatusEntries()
    setSelectedItems([])
    props.onRefresh?.()
  }

  const displayItems = () => {
    return tree.getSelectedItems().map((e) => {
      const data = e.getItemData()
      const model = fromStatusEntry(data, false, workingCopy.path)
      return { ...model, path: data.path }
    })
  }

  const operationHandler = new OperationHandler(modal, refresh)

  const showInfo = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        const options: InfoOptions = {
          path: tree.getSelectedItems()[0].getItemData().path,
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

    operationHandler.showPatchDialog(file, tree.getSelectedItems()[0].getItemData().path)
  }

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: t('shared.action.refresh'),
      // onClick: refresh,
      enable: state.refresh,
      children: <RefreshIcon></RefreshIcon>,
      async onClick() {
        await refresh()
      },
    },
    {
      tooltip: t('shared.action.add'),
      // onClick: () => setAddDialogKey((i) => i + 1),
      enable: state.add,
      children: <OperationAddIcon></OperationAddIcon>,
      onClick() {
        operationHandler.showAddDialog(displayItems())
      },
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
          tree.getSelectedItems()[0].getItemData().path,
        )
      },
    },
    {
      sync: false,
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
      onClick: () => operationHandler.showMkdirDialog(selectedItems[0]),
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
      onClick: () =>
        operationHandler.showSwitchDialog(tree.getSelectedItems()[0].getItemData().path),
    },
    {
      tooltip: t('shared.action.merge'),
      enable: state.merge,
      children: <OperationMergeIcon />,
      onClick: () =>
        operationHandler.showMergeDialog(tree.getSelectedItems()[0].getItemData().path, {
          peg: { source: '', pegRevision: 'working', rangesToMerge: null },
        }),
    },
    {
      tooltip: t('shared.action.export'),
      enable: state.export,
      children: <OperationExportIcon />,
      onClick: () =>
        operationHandler.showExportDialog(tree.getSelectedItems()[0].getItemData().path),
    },
  ]
  const bar = (
    <div className={cx(flex, items_center, !props.visible && hidden)}>
      <OperationBar className={cx(gap_x_1, flex_1)} size={25} icons={icons}></OperationBar>
    </div>
  )

  const [showAll, setShowAll] = useState(false)
  // const [statusEntries, setStatusEntries] = useState<StatusEntry[]>([])
  const refreshStatusEntries = async () => {
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
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
        const result = await context.status(options)
        props.onStatusEntriesChanged?.(result.entries)
        // setStatusEntries(result.entries)
      },
    })
  }
  useEffect(() => {
    if (props.statusEntries !== undefined) {
      return
    }
    refreshStatusEntries()
  }, [])

  useEffect(() => {
    if (tree.getSelectedItems().length === 1) {
      const items = tree.getSelectedItems()
      props.onSelected?.(items[0].getItemData().path)
    } else if (tree.getSelectedItems().length === 0) {
      props.onSelected?.(null)
    }
  }, [selectedItems])

  // 一次性构建所有可见节点的路径集合：
  // 对每个 status entry 向上收集其全部祖先前缀，判断某节点是否可见即为 O(1) 查表，
  // 避免此前每个节点都对 statusEntries 做一次线性前缀扫描（O(n^2)）。
  const visiblePaths = useMemo(() => {
    const paths = new Set<string>()
    for (const entry of props.statusEntries ?? []) {
      let path: string | null = entry.path
      while (path !== null && path !== '') {
        if (paths.has(path)) {
          break
        }
        paths.add(path)
        const parent = localPath.getParent(path)
        if (parent === null || parent === path) {
          break
        }
        path = parent
      }
    }
    return paths
  }, [props.statusEntries])

  const itemIsVisible = (path: string) => path === '' || visiblePaths.has(path)

  const optionBar = (
    <div className={cx(!props.visible && hidden, flex_1, flex, items_center)}>
      <Checkbox checked={showAll} onChange={(e) => setShowAll(e.target.checked ?? false)}>
        {t('changes.tree.all')}
      </Checkbox>
      <div className={cx(flex_1)}></div>
      <OperationBar
        className={cx(gap_x_1)}
        icons={[
          {
            tooltip: t('shared.action.expand'),
            async onClick() {
              await tree.expandAll()
            },
            enable: true,
            children: <TreeExpandIcon></TreeExpandIcon>,
            sync: false,
          },
          {
            tooltip: t('shared.action.collapse'),
            onClick() {
              tree.collapseAll()
            },
            enable: true,
            children: <TreeCollapseIcon></TreeCollapseIcon>,
          },
        ]}
      ></OperationBar>
    </div>
  )

  const treeItems = tree.getItems()
  const visibleItems = showAll
    ? treeItems
    : treeItems.filter((item) => itemIsVisible(item.getItemData().path))

  const virtualizer = useVirtualizer({
    count: visibleItems.length,
    getScrollElement: () => {
      if (!isReady) return null
      return scrollRef.current
    },
    estimateSize: () => 30,
    overscan: 5,
    getItemKey: (index) => visibleItems[index].getId(),
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

  const ignoreItems: ContextMenuItemModel[] = []

  const selection = tree.getSelectedItems()

  let extension = null
  if (state.ignore) {
    extension = repoPath.getExtension(selection[0].getItemData().path)
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
          if (selection.length === 0) {
            return
          }
          addIgnoreLine(selection[0].getItemData().path, `*.${extension}`, false)
        },
      },
    })
    ignoreItems.push({
      item: {
        content: t('changes.ignore.extensionRecursively', { extension }),
        onSelect: () => {
          if (selection.length === 0) {
            return
          }
          addIgnoreLine(selection[0].getItemData().path, `*.${extension}`, true)
        },
      },
    })
  }

  if (state.ignore) {
    const fileName = repoPath.getFileName(selection[0].getItemData().path)

    if (fileName) {
      ignoreItems.push({
        item: {
          content: t('changes.ignore.name', { name: fileName }),
          onSelect: () => {
            if (selection.length === 0) {
              return
            }
            addIgnoreLine(selection[0].getItemData().path, fileName, false)
          },
        },
      })
      ignoreItems.push({
        item: {
          content: t('changes.ignore.nameRecursively', { name: fileName }),
          onSelect: () => {
            if (selection.length === 0) {
              return
            }
            addIgnoreLine(selection[0].getItemData().path, fileName, true)
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
        onSelect: () => operationHandler.showAddDialog(displayItems()),
      },
    },
    {
      item: {
        content: t('shared.action.update'),
        disabled: !state.update,
        onSelect: () => operationHandler.showUpdateDialog(displayItems()),
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
            tree.getSelectedItems()[0].getItemData().path,
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
        onSelect: () =>
          operationHandler.showMkdirDialog(tree.getSelectedItems()[0].getItemData().path),
      },
    },
    {
      item: {
        content: t('shared.action.export'),
        disabled: !state.export,
        onSelect: () =>
          operationHandler.showExportDialog(tree.getSelectedItems()[0].getItemData().path),
      },
    },
    {
      item: {
        content: t('shared.action.switch'),
        disabled: !state.switch,
        onSelect: () =>
          operationHandler.showSwitchDialog(tree.getSelectedItems()[0].getItemData().path),
      },
    },
    {
      item: {
        content: t('shared.action.merge'),
        disabled: !state.merge,
        onSelect: () =>
          operationHandler.showMergeDialog(tree.getSelectedItems()[0].getItemData().path, {
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
          const selection = tree.getSelectedItems()
          copyMove.setSource({
            copy: selection.map((e) => ({
              path: e.getItemData().path,
              pegRevision: 'working',
              revision: 'working',
              nodeKind: e.getItemData().nodeKind,
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
          const selection = tree.getSelectedItems()
          copyMove.setSource({
            isLocal: true,
            move: selection.map((e) => ({
              path: e.getItemData().path,
              nodeKind: e.getItemData().nodeKind,
            })),
          })
        },
      },
    },
    {
      item: {
        content: t('shared.action.rename'),
        disabled: !state.rename,
        onSelect: () => {
          const item = tree.getSelectedItems()[0].getItemData()

          modal.show(NiceMoveRenameDialog, {
            path: item.path,
          })
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
            copyMove.source.isLocal === false),
        onSelect: () => {
          if (copyMove.source !== undefined) {
            if ('copy' in copyMove.source) {
              modal.show(NiceCopyDialog, {
                needCommitMessage: false,
                sources: copyMove.source.copy,
                defaultDestination: tree.getSelectedItems()[0].getItemData().path,
              })
            } else if ('move' in copyMove.source) {
              modal.show(NiceMoveDialog, {
                sources: copyMove.source.move,
                needCommitMessage: false,
              })
            }
          }
        },
      },
    },
    {
      item: {
        content: t('shared.action.fileHistory'),
        disabled: !state.history,
        onSelect: async () => {
          Logger.info(
            'Open file history: ',
            tree.getSelectedItems().map((e) => e.getItemData()),
          )
          operationHandler.showFileHistoryDialog(tree.getSelectedItems()[0].getItemData())
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
        disabled: tree.getSelectedItems().length !== 1,
        onSelect: () => {
          const selected = tree.getSelectedItems()[0].getItemData()
          writeText(selected.path)
        },
      },
    },
    {
      item: {
        content: t('shared.action.copyRelativePath'),
        disabled: tree.getSelectedItems().length !== 1,
        onSelect: () => {
          const selected = tree.getSelectedItems()[0].getItemData()
          const path = repoPath.stripPrefix(selected.path, workingCopy.path)
          if (path) {
            writeText(path)
          }
        },
      },
    },
    {
      item: {
        content: t('workspace.changes.revealInFolder'),
        disabled: selection.length === 0,
        onSelect: async () => {
          const paths = selection.map((e) => e.getItemData().path).filter((p) => p !== '')
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
          selection.length !== 0 &&
          selection[0].getItemData().path !== workspace.path,
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
  ]

  return (
    <div className={cx(flex_1, flex, !props.visible && hidden)}>
      <div className={cx(flex_1, overflow_y_auto)} ref={scrollRef}>
        <div
          {...tree.getContainerProps()}
          style={{
            height: virtualizer.getTotalSize(),
            position: 'relative',
            width: '100%',
          }}
        >
          {virtualizer.getVirtualItems().map((virtualItem) => {
            const item = visibleItems[virtualItem.index]
            const props = item.getProps()
            const level = item.getItemMeta().level
            const isFolder = item.isFolder()
            const isExpanded = item.isExpanded()
            const isSelected = item.isSelected()
            const model = fromStatusEntry(item.getItemData(), false)

            return (
              <ContextMenu menu={menu}>
                <div
                  {...props}
                  key={virtualItem.key}
                  data-index={virtualItem.index}
                  ref={(r) => {
                    virtualizer.measureElement(r)
                    props.ref(r)
                  }}
                  className={cx(treeNode, isSelected && treeNodeSelected)}
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
                  <span className={cx(expandIcon, isFolder && !isExpanded && expandIconCollapsed)}>
                    <IconTreeTriangleDown
                      className={cx(!isFolder && visibility_hidden)}
                      size="default"
                      onClick={item.isExpanded() ? item.collapse : item.expand}
                    />
                  </span>
                  <WorkingCopyItem
                    {...model}
                    className={cx(overflow_hidden, whitespace_nowrap, flex_1)}
                  ></WorkingCopyItem>
                </div>
              </ContextMenu>
            )
          })}
        </div>
      </div>
      {props.optionBarContainer !== null && createPortal(optionBar, props.optionBarContainer)}
      {workingCopy.changesViewOperationContainer !== null &&
        createPortal(bar, workingCopy.changesViewOperationContainer)}
    </div>
  )
}
