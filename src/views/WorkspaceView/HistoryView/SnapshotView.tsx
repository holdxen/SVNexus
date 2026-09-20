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
import { css, cx } from '@linaria/core'
import { useVirtualizer } from '@tanstack/react-virtual'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { useEffect, useMemo, useRef, useState } from 'react'
import { Group, Panel, Separator } from 'react-resizable-panels'
import * as uuid from 'uuid'

import { CatOptions } from '@/bindings/CatOptions'
import { ListEntry } from '@/bindings/ListEntry'
import { ListOptions } from '@/bindings/ListOptions'
import { ContextMenu, ContextMenuItemModel } from '@/components/ContextMenu'
import StrongEditor, { StrongEditorRef } from '@/components/StrongEditor'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { base64Encode } from '@/context/Functions'
import { Subversion, useSubversion } from '@/context/Subversion'
import {
  flex_1,
  min_w_0,
  flex,
  hidden,
  items_center,
  p_1,
  gap_x_1,
  visibility_hidden,
  min_h_0,
  whitespace_nowrap,
  overflow_hidden,
  overflow_y_auto,
} from '@/styles/Classes'
import Logger from '@/utils/Logger'
import { combineUrl } from '@/utils/Url'

export interface SnapshotViewProps {
  // url: string

  root: string
  pegRevision: number
  revision: number
  className?: string
  location: string
}

// function testEntry(): ListEntry {
//   return {
//     path: '',
//     kind: 'file',
//     lastAuthor: '',
//     size: 0,
//     time: 0,
//     lock: null,
//     hasProperties: false,
//     createdRevision: 0,
//     absolutePath: '',
//     externalParentUrl: null,
//     externalTarget: null,
//   }
// }

// type Entry = ListEntry & {
//   name: string
// }

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
    width: 16px;
    height: 16px;
  }
`

export function SnapshotView(props: SnapshotViewProps) {
  // Logger.info("Snap shot view:", props.url)
  const subversion = useSubversion()
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const editor = useRef<StrongEditorRef | null>(null)
  const [currentEditor, setCurrentEditor] = useState<string>()
  const [isLoading, setIsLoading] = useState(false)

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
      if (path === '') {
        return '/'
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
              fetchLocks: false,
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
                // const name = await pathGetFileName(i.absolutePath)
                // Logger.info('item is', itemId, ' path is', i.path)
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
          fetchLocks: false,
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
    if (selectedItems.length === 0) {
      setCurrentEditor(undefined)
    } else if (selectedItems.length === 1) {
      const item = tree.getItemInstance(selectedItems[0])
      if (item.getItemData().kind === 'file') {
        setCurrentEditor(selectedItems[0])
        // const path = item.getItemData().absolutePath + '/' + item.getItemData().path
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

                  const menu: ContextMenuItemModel[] = []
                  if (item.getItemData().kind === 'file') {
                    menu.push({
                      item: {
                        content: 'Save',
                        onSelect: async () => {
                          const result = await save({
                            title: '保存文件',
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
                                  content: `Save ${result} successfully`,
                                  stack: true,
                                })
                              },
                              onError: (e) => {
                                Toast.error({
                                  content: `Save ${result} failed: ${e}`,
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
                        content: 'File history',
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
                      <div className={cx(svg, flex, items_center)}>
                        <FileKindIcon kind={item.getItemData().kind ?? 'unknown'}></FileKindIcon>
                      </div>
                      <span>{item.getItemName()}</span>
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
