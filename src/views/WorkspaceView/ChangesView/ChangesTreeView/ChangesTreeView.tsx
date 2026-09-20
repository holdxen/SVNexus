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
import { cx, css } from '@linaria/core'
import { useVirtualizer } from '@tanstack/react-virtual'
import { open } from '@tauri-apps/plugin-dialog'
import { useEffect, useMemo, useRef, useState } from 'react'
import { createPortal } from 'react-dom'

import { InfoOptions } from '@/bindings/InfoOptions'
import { StatusEntry } from '@/bindings/StatusEntry'
import { StatusOptions } from '@/bindings/StatusOptions'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import { Subversion, useSubversion } from '@/context/Subversion'
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
import { localPath } from '@/utils/Path'

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
  }
}

export function ChangesTreeView(props: ChangesTreeViewProps) {
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const [loadingItemData, setLoadingItemData] = useState<string[]>([])
  const [loadingItemChildrens, setLoadingItemChildrens] = useState<string[]>([])

  const workingCopy = useWorkingCopyContext()
  const subversion = useSubversion()

  const loadingItem: TreeEntry = {
    path: '',
    name: '加载中 > ...',
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
          throw new Error('Empty working copy')
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
        Logger.info('on get tree children', itemId)
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

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: 'Refresh',
      // onClick: refresh,
      enable: state.refresh,
      children: <RefreshIcon></RefreshIcon>,
      async onClick() {
        await refresh()
      },
    },
    {
      tooltip: 'Add',
      // onClick: () => setAddDialogKey((i) => i + 1),
      enable: state.add,
      children: <OperationAddIcon></OperationAddIcon>,
      onClick() {
        operationHandler.showAddDialog(displayItems())
      },
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
          tree.getSelectedItems()[0].getItemData().path,
        )
      },
    },
    {
      sync: false,
      tooltip: 'Patch',
      enable: state.patch,
      children: <OperationPatchIcon></OperationPatchIcon>,
      async onClick() {
        const file = await open({
          multiple: false,
          directory: false,
        })
        if (file === null) {
          return
        }

        operationHandler.showPatchDialog(file, selectedItems[0])
      },
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
      onClick: () => operationHandler.showMkdirDialog(selectedItems[0]),
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
              path: selectedItems[0],
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
      onClick: () =>
        operationHandler.showSwitchDialog(tree.getSelectedItems()[0].getItemData().path),
    },
    {
      tooltip: 'Merge',
      enable: state.merge,
      children: <OperationMergeIcon />,
      onClick: () =>
        operationHandler.showMergeDialog(tree.getSelectedItems()[0].getItemData().path, {
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
  const [statusEntries, setStatusEntries] = useState<StatusEntry[]>([])
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
        setStatusEntries(result.entries)
      },
    })
  }
  useEffect(() => {
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
    for (const entry of statusEntries) {
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
  }, [statusEntries])

  const itemIsVisible = (path: string) => path === '' || visiblePaths.has(path)

  const optionBar = (
    <div className={cx(!props.visible && hidden, flex_1, flex, items_center)}>
      <Checkbox checked={showAll} onChange={(e) => setShowAll(e.target.checked ?? false)}>
        All
      </Checkbox>
      <div className={cx(flex_1)}></div>
      <OperationBar
        className={cx(gap_x_1)}
        icons={[
          {
            tooltip: 'Expand',
            async onClick() {
              await tree.expandAll()
            },
            enable: true,
            children: <TreeExpandIcon></TreeExpandIcon>,
            sync: false,
          },
          {
            tooltip: 'Collapse',
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
            )
          })}
        </div>
      </div>
      {props.optionBarContainer === null ? (
        <></>
      ) : (
        createPortal(optionBar, props.optionBarContainer)
      )}
      {workingCopy.changesViewOperationContainer === null ? (
        <></>
      ) : (
        createPortal(bar, workingCopy.changesViewOperationContainer)
      )}
    </div>
  )
}
