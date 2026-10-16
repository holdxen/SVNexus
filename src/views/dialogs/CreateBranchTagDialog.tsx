'use no memo'

import { IconTreeTriangleDown } from '@douyinfe/semi-icons'
import { Card, Spin, Toast } from '@douyinfe/semi-ui'
import {
  asyncDataLoaderFeature,
  buildProxiedInstance,
  FeatureImplementation,
  hotkeysCoreFeature,
  selectionFeature,
} from '@headless-tree/core'
import { useTree } from '@headless-tree/react'
import { css, cx } from '@linaria/core'
import { useVirtualizer } from '@tanstack/react-virtual'
import { useEffect, useLayoutEffect, useMemo, useRef, useState } from 'react'

import { CopyOptions } from '@/bindings/CopyOptions'
import { CopySourceItem } from '@/bindings/CopySourceItem'
import { InfoOptions } from '@/bindings/InfoOptions'
import { ListEntry } from '@/bindings/ListEntry'
import { ListOptions } from '@/bindings/ListOptions'
import { Revision } from '@/bindings/Revision'
import PureInput from '@/components/PureInput'
import PureTextArea from '@/components/PureTextArea'
import { ScrollArea } from '@/components/ScrollArea'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  absolute,
  block_minimizable,
  border_box,
  break_all,
  break_word,
  flex,
  flex_1,
  flex_col,
  gap_x_1,
  gap_y_2,
  inset_0,
  items_center,
  justify_center,
  min_h_0,
  minimizable,
  overflow_hidden,
  overflow_x_auto,
  overflow_y_auto,
  p_1,
  p_2,
  relative,
  select_text,
  text_12px,
  visibility_hidden,
  whitespace_nowrap,
} from '@/styles/Classes'
import errorHumanString, { errorIsAccessDenied } from '@/utils/Error'
import { repoPath } from '@/utils/Path'
import { combineUrl } from '@/utils/Url'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface CreateBranchTagDialogProps {
  afterClose?: () => void
  onCancel?: () => void
  onOK?: () => void
  visible: boolean

  /** 复制源：路径或 URL 加版本，会原样交给 svn copy */
  source: CopySourceItem
  /** 预填的基目录 URL */
  baseUrl?: string
  /** 预填的分支/标签名称 */
  name?: string
}

export function NiceCreateBranchTagDialog(props: {
  source: CopySourceItem
  baseUrl?: string
  name?: string
}) {
  const modal = useCurrentModal()

  return (
    <CreateBranchTagDialog
      afterClose={modal.remove}
      visible={modal.visible}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOK={() => {
        modal.resolve(true)
        modal.hide()
      }}
      {...props}
    ></CreateBranchTagDialog>
  )
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

const treeBox = css`
  height: 150px;
`

function listOptions(path: string, depth: ListOptions['depth']): ListOptions {
  return {
    path,
    pegRevision: 'head',
    revision: 'head',
    patterns: null,
    depth,
    direntCreatedRevision: false,
    direntHasProperties: false,
    direntKind: true,
    direntLastAuthor: false,
    direntSize: false,
    direntTime: false,
    fetchLocks: false,
    includeExternals: false,
  }
}

/**
 * 「能浏览到的最上层目录」按仓库 UUID 缓存。
 *
 * 它是权限配置的属性，和具体源路径无关——同一仓库里换个源进来答案是一样的，
 * 所以整个会话探测一次就够（本次会话内有效，不落盘）。
 */
const browseRootCache = new Map<string, string>()

/** 仓库根到目标 URL 的逐层候选链（含两端），用于逐层向下找第一个可读的层级。 */
function ancestorChain(root: string, target: string): string[] {
  const base = root.endsWith('/') ? root.slice(0, -1) : root
  if (base === '') {
    return []
  }
  if (target !== base && !target.startsWith(`${base}/`)) {
    return [base]
  }

  const chain = [base]
  let current = base
  for (const segment of target.slice(base.length).split('/')) {
    if (segment === '') {
      continue
    }
    current = `${current}/${segment}`
    chain.push(current)
  }

  return chain
}

function revisionLabel(revision: Revision): string {
  if (typeof revision === 'string') {
    return revision
  }
  if ('number' in revision) {
    return `r${revision.number}`
  }
  return new Date(revision.date).toLocaleString()
}

interface FolderTreeProps {
  rootUrl: string
  onSelect: (url: string) => void
}

/**
 * 仓库目录树：只列文件夹，懒加载。节点 id 就是该目录的 URL，
 * 所以选中项的 id 可以直接当作基目录地址使用。
 */
function FolderTree(props: FolderTreeProps) {
  const subversion = useSubversion()
  const [selectedItems, setSelectedItems] = useState<string[]>([])
  const [isRootLoading, setIsRootLoading] = useState(true)
  const [contentWidth, setContentWidth] = useState(0)
  const [isReady, setIsReady] = useState(false)
  const scrollRef = useRef<HTMLDivElement>(null)
  const virtualizerRef = useRef<any>(null)

  const doubleClickBehavior: FeatureImplementation = {
    itemInstance: {
      getProps: ({ item, prev }) => ({
        ...prev?.(),
        onDoubleClick: () => {
          item.setFocused()

          if (!item.isFolder()) {
            return
          }

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

  const tree = useTree<ListEntry>({
    instanceBuilder: buildProxiedInstance,
    scrollToItem: (item) => {
      virtualizerRef.current?.scrollToIndex(item.getItemMeta().index)
    },
    state: {
      selectedItems,
    },
    setSelectedItems: (items) => {
      const next = typeof items === 'function' ? items(selectedItems) : items
      setSelectedItems(next)
      if (next.length > 0) {
        props.onSelect(next[0])
      }
    },
    initialState: {
      expandedItems: [props.rootUrl],
      selectedItems: [],
    },
    getItemName: (item) => {
      const path = item.getItemData().path
      if (path !== '') {
        return path
      }
      return repoPath.getFileName(item.getItemMeta().itemId) ?? '/'
    },
    isItemFolder: (item) => {
      return item.getItemData().kind === 'directory'
    },
    dataLoader: {
      getChildrenWithData: async (itemId: string) => {
        const result = await Subversion.callOnce({
          factory: subversion,
          call: async (context) => {
            const entries = (await context.list(listOptions(itemId, 'immediates'))).entries

            return entries
              .filter((e) => e.kind === 'directory' && e.path !== '')
              .sort((a, b) => a.path.localeCompare(b.path))
              .map((e) => ({ id: combineUrl(itemId, e.path), data: e }))
          },
        })
        if (itemId === props.rootUrl) {
          setIsRootLoading(false)
        }
        return result ?? []
      },
      getItem: async (itemId: string) => {
        const result = await Subversion.callOnce({
          factory: subversion,
          call: async (context) => {
            const entries = (await context.list(listOptions(itemId, 'empty'))).entries
            if (entries.length !== 1) {
              throw new Error(`expected exactly one entry, got ${entries.length}`)
            }
            return entries[0]
          },
        })
        if (result === null) {
          throw new Error(`Failed to get itemId:${itemId}`)
        }
        return result
      },
    },
    rootItemId: props.rootUrl,
    features: [doubleClickBehavior, asyncDataLoaderFeature, selectionFeature, hotkeysCoreFeature],
  })

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

  // 行是绝对定位、宽度跟着容器走的，路径一深就会被裁掉。名字那个 span 不会收缩（nowrap +
  // min-width:auto），所以「内边距 + 列间距 + 各子元素实际宽度」算出来的值与容器宽度无关，
  // 可以放心当需求宽度用：只有它真的超出可视宽度时才撑宽容器，让外层出现横向滚动条。
  // 用与容器无关的量，才不会出现「撑宽后不再超出 → 又缩回去」的来回抖动。
  useLayoutEffect(() => {
    const element = scrollRef.current
    if (element === null) {
      return
    }

    const available = element.clientWidth
    let required = 0

    for (const row of element.querySelectorAll<HTMLElement>('[data-index]')) {
      const style = getComputedStyle(row)
      const gap = parseFloat(style.columnGap) || 0
      let width = parseFloat(style.paddingLeft) + parseFloat(style.paddingRight)
      for (const child of Array.from(row.children)) {
        width += child.getBoundingClientRect().width
      }
      width += gap * Math.max(0, row.children.length - 1)
      required = Math.max(required, width)
    }

    // 留 1px 容差，免得浮点取整凑出一个没有任何内容被裁的滚动条
    const next = required > available + 1 ? Math.ceil(required) : 0
    if (next !== contentWidth) {
      setContentWidth(next)
    }
  })

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

    return () => {
      observer.disconnect()
    }
  }, [])

  return (
    <div
      className={cx(relative, flex_1, min_h_0, overflow_x_auto, overflow_y_auto)}
      ref={scrollRef}
    >
      {isRootLoading && (
        <div className={cx(absolute, inset_0, flex, items_center, justify_center)}>
          <Spin size="small"></Spin>
        </div>
      )}
      <div
        {...tree.getContainerProps()}
        style={{
          height: virtualizer.getTotalSize(),
          minWidth: contentWidth === 0 ? undefined : contentWidth,
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

          return (
            <div
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
                    onClick={(e) => {
                      // 只展开/收起，不改变选中的基目录
                      e.stopPropagation()
                      if (item.isExpanded()) {
                        item.collapse()
                      } else {
                        item.expand()
                      }
                    }}
                  />
                </span>
              </Spin>
              <div className={cx(svg, flex, items_center)}>
                <FileKindIcon kind={item.getItemData().kind ?? 'unknown'}></FileKindIcon>
              </div>
              <span>{item.getItemName()}</span>
            </div>
          )
        })}
      </div>
    </div>
  )
}

export default function CreateBranchTagDialog(props: CreateBranchTagDialogProps) {
  const t = useT()
  const subversion = useSubversion()

  const [baseUrl, setBaseUrl] = useState(props.baseUrl ?? '')
  const [name, setName] = useState(props.name ?? '')
  const [commitMessage, setCommitMessage] = useState('')

  const [sourceUrl, setSourceUrl] = useState<string | null>(null)
  const [repositoryRootUrl, setRepositoryRootUrl] = useState<string | null>(null)
  const [repositoryUuid, setRepositoryUuid] = useState<string | null>(null)
  const [browseRootUrl, setBrowseRootUrl] = useState<string | null>(null)
  const [browseError, setBrowseError] = useState<string | null>(null)
  const [error, setError] = useState<string | null>(null)

  // 一次 info 同时拿到源 URL（显示用）和仓库根 URL（目录树的根）
  useEffect(() => {
    if (!props.visible) {
      return
    }

    let cancelled = false

    setError(null)
    setSourceUrl(null)
    setRepositoryRootUrl(null)
    setRepositoryUuid(null)
    setBrowseRootUrl(null)
    setBrowseError(null)

    const load = async () => {
      const result = await Subversion.callOnce({
        factory: subversion,
        onError: (e) => {
          if (!cancelled) {
            setError(errorHumanString(e))
          }
        },
        call: async (context) => {
          const options: InfoOptions = {
            path: props.source.path,
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
            throw new Error(t('svnDialogs.branchTag.notVersioned', { path: props.source.path }))
          }
          return {
            sourceUrl: entry.url,
            repositoryRootUrl: entry.repositoryRootUrl,
            repositoryUuid: entry.repositoryUuid,
          }
        },
      })

      if (cancelled || result === null) {
        return
      }

      setSourceUrl(result.sourceUrl)
      setRepositoryRootUrl(result.repositoryRootUrl)
      setRepositoryUuid(result.repositoryUuid)
    }

    load()

    return () => {
      cancelled = true
    }
  }, [props.visible, props.source.path])

  // 没有仓库根读权限的用户，也要能在自己有权的最上层目录里浏览、挑目标目录：
  // 从仓库根沿源 URL 逐层向下试，停在第一个可读的层级。
  // 候选链的最后一层就是源 URL 本身，而 info 已经在它上面成功过（info 和 list 都要求读权限），
  // 所以这个循环一定会成功，走不到「全都不行」那一步。
  useEffect(() => {
    if (!props.visible || repositoryRootUrl === null || sourceUrl === null) {
      return
    }

    let cancelled = false

    setBrowseRootUrl(null)
    setBrowseError(null)

    const probe = async () => {
      let lastError: any = null

      const check = async (path: string): Promise<'ok' | 'denied' | 'error'> => {
        let outcome: 'ok' | 'denied' | 'error' = 'ok'

        await Subversion.callOnce({
          factory: subversion,
          // 传了 onError 才不会为每一层被拒都弹一个 Toast（callOnce 默认会弹）
          onError: (e) => {
            lastError = e
            outcome = errorIsAccessDenied(e) ? 'denied' : 'error'
          },
          call: async (context) => {
            await context.list(listOptions(path, 'empty'))
            return true
          },
        })

        return outcome
      }

      const cached = repositoryUuid === null ? undefined : browseRootCache.get(repositoryUuid)
      const chain = ancestorChain(repositoryRootUrl, sourceUrl)
      // 命中缓存就只花一次请求
      const candidates = cached !== undefined && chain.includes(cached) ? [cached] : chain

      for (const candidate of candidates) {
        const outcome = await check(candidate)
        if (cancelled) {
          return
        }
        if (outcome === 'ok') {
          if (repositoryUuid !== null) {
            browseRootCache.set(repositoryUuid, candidate)
          }
          setBrowseRootUrl(candidate)
          return
        }
        if (outcome === 'denied') {
          continue
        }

        setBrowseError(errorHumanString(lastError))
        return
      }

      setBrowseError(t('svnDialogs.branchTag.noReadableRoot'))
    }

    probe()

    return () => {
      cancelled = true
    }
  }, [props.visible, repositoryRootUrl, sourceUrl, repositoryUuid])

  const destination = useMemo(() => {
    const base = baseUrl.trim()
    const suffix = name.trim()
    if (base === '' || suffix === '') {
      return ''
    }
    return combineUrl(base, suffix)
  }, [baseUrl, name])

  const onOk = async () => {
    await Subversion.callOnce({
      factory: subversion,
      onError: (e) => {
        Toast.error({
          content: t('svnDialogs.branchTag.failed', { error: errorHumanString(e) }),
          stack: true,
        })
      },
      async call(context) {
        if (baseUrl.trim() === '') {
          Toast.error({
            content: t('svnDialogs.branchTag.baseRequired'),
            stack: true,
          })
          return
        }
        if (name.trim() === '') {
          Toast.error({
            content: t('svnDialogs.branchTag.nameRequired'),
            stack: true,
          })
          return
        }
        if (commitMessage === '') {
          Toast.error({
            content: t('shared.error.commitMessageRequired'),
            stack: true,
          })
          return
        }
        if (destination === '') {
          return
        }

        // 建分支/标签是服务端操作：源一律用 info 解析出来的 URL，
        // 这样 svn 走的是纯 URL→URL 的服务端复制，不会牵扯到工作副本。
        const source: CopySourceItem = {
          path: sourceUrl ?? props.source.path,
          pegRevision: props.source.pegRevision,
          revision: props.source.revision,
        }

        const options: CopyOptions = {
          sources: [source],
          destination,
          copyAsChild: false,
          // 基目录可以手填一个还不存在的路径，交给服务端一并建出来
          makeParents: true,
          ignoreExternals: false,
          metadataOnly: false,
          pinExternals: false,
          revisionPropertyTable: null,
          commitMessage,
        }
        const result = await context.copy(options)

        Toast.success({
          content:
            result.commitInfo === null
              ? t('svnDialogs.branchTag.createdNoRevision', { url: destination })
              : t('svnDialogs.branchTag.created', { revision: result.commitInfo.revision }),
          stack: true,
        })
        props.onOK?.()
      },
    })
  }

  const readOnlyLine = (content: string) => (
    <span
      className={cx(flex_1, text_12px, break_all, break_word, select_text)}
      style={{ color: 'var(--semi-color-text-2)' }}
    >
      {content}
    </span>
  )

  return (
    <Dialog
      afterClose={props.afterClose}
      onCancel={props.onCancel}
      onOk={onOk}
      size="medium"
      title={t('shared.action.createBranchTag')}
      visible={props.visible}
    >
      <ScrollArea
        className={cx(flex, flex_1, minimizable, border_box)}
        contentClassName={cx(flex_1, flex, block_minimizable, flex_col, gap_y_2)}
      >
        <p className={cx(text_12px)} style={{ margin: 0, color: 'var(--semi-color-text-2)' }}>
          {t('svnDialogs.branchTag.description')}
        </p>
        <DialogFormItem title={t('svnDialogs.branchTag.source')}>
          {readOnlyLine(
            `${sourceUrl ?? props.source.path} @ ${revisionLabel(props.source.revision)}`,
          )}
        </DialogFormItem>
        <DialogFormItem title={t('svnDialogs.branchTag.repositoryTree')}>
          <Card
            className={cx(treeBox, flex_1, flex, flex_col)}
            bodyStyle={{
              display: 'flex',
              flex: 1,
              minHeight: 0,
              padding: '0.5rem',
            }}
          >
            {error !== null || browseError !== null ? (
              <div
                className={cx(
                  flex_1,
                  min_h_0,
                  overflow_y_auto,
                  text_12px,
                  break_all,
                  break_word,
                  select_text,
                )}
                style={{ color: 'var(--semi-color-danger)' }}
              >
                {error ?? browseError}
              </div>
            ) : browseRootUrl === null ? (
              <div className={cx(flex_1, flex, items_center, justify_center, p_2)}>
                <Spin size="small"></Spin>
              </div>
            ) : (
              <FolderTree rootUrl={browseRootUrl} onSelect={setBaseUrl}></FolderTree>
            )}
          </Card>
        </DialogFormItem>
        <DialogFormItem title={t('svnDialogs.branchTag.baseUrl')}>
          <PureInput
            className={cx(flex_1)}
            value={baseUrl}
            onChange={(value) => setBaseUrl(value)}
          ></PureInput>
        </DialogFormItem>
        <DialogFormItem title={t('svnDialogs.branchTag.name')}>
          <PureInput
            className={cx(flex_1)}
            value={name}
            onChange={(value) => setName(value)}
          ></PureInput>
        </DialogFormItem>
        <DialogFormItem title={t('svnDialogs.branchTag.destination')}>
          <span className={cx(flex_1, break_all, break_word, select_text)}>
            {destination === '' ? '—' : destination}
          </span>
        </DialogFormItem>
        <DialogFormItem title={t('svnDialogs.branchTag.commitMessage')}>
          <PureTextArea
            className={cx(flex_1)}
            value={commitMessage}
            onChange={(value) => setCommitMessage(value)}
          ></PureTextArea>
        </DialogFormItem>
      </ScrollArea>
    </Dialog>
  )
}
