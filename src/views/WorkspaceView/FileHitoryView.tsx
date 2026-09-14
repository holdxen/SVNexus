import UserAvatarIcon from '@icons/UserAvatar.svg?react'
import { cx } from '@linaria/core'
import { VirtualItem } from '@tanstack/react-virtual'
import { useEffect, useRef, useState } from 'react'
import { Group, Panel, Separator } from 'react-resizable-panels'
import { useParams } from 'react-router'

import { LogEntry } from '@/bindings/LogEntry'
import { Revision } from '@/bindings/Revision'
import { RevisionPropertyName } from '@/bindings/RevisionPropertyName'
import StrongDifferenceEditor, {
  StrongDifferenceEditorRef,
} from '@/components/StrongDifferenceEditor'
import VirtualList from '@/components/VirtualList'
import { base64Decode } from '@/context/Functions'
import { ModalProvider } from '@/lib/multi-modal'
import {
  flex,
  flex_1,
  flex_col,
  gap_x_1,
  gap_y_1,
  h_full,
  min_h_0,
  overflow_hidden,
  w_full,
  whitespace_nowrap,
} from '@/styles/Classes'

import { SubversionProvider } from '../SubversionProvider'

export interface FileHistoryViewProps {
  showDifference: boolean
  url: string
  pegRevision: Revision
  className?: string
}

export function FileHistoryItem({ entry }: { entry: LogEntry }) {
  const authorKey: RevisionPropertyName = 'svn:author'
  const dateKey: RevisionPropertyName = 'svn:date'
  const logKey: RevisionPropertyName = 'svn:log'

  const author = entry.revisionProperties?.[authorKey]
  const date = entry.revisionProperties?.[dateKey]
  const log = entry.revisionProperties?.[logKey]

  return (
    <div className={cx(flex, flex_col)}>
      <div className={cx(flex, gap_x_1)}>
        <UserAvatarIcon style={{ width: 20, height: 20 }} />
        <div>{author}</div>
        <div className={cx(flex_1)}></div>
        <div>{date}</div>
        <div>{`r${entry.revision}`}</div>
      </div>
      <div className={cx(whitespace_nowrap, overflow_hidden)}>{log}</div>
    </div>
  )
}

export function RouteFileHistoryView() {
  const { url, revision } = useParams()

  const [decodeUrl, setDecodeUrl] = useState<string | null>(null)

  useEffect(() => {
    const call = async () => {
      if (url) {
        try {
          const decoded = await base64Decode(url)
          setDecodeUrl(new TextDecoder('utf-8').decode(decoded))
        } catch (error) {}
      }
    }
    call()
  }, [url])

  if (decodeUrl && revision) {
    return (
      <ModalProvider>
        <SubversionProvider className={cx(h_full, w_full)} singleton={false}>
          <FileHistoryView
            className={cx(flex_1)}
            url={decodeUrl}
            pegRevision={{ number: Number(revision) }}
            showDifference
          ></FileHistoryView>
        </SubversionProvider>
      </ModalProvider>
    )
  } else {
    return <></>
  }
}

export default function FileHistoryView(props: FileHistoryViewProps) {
  // const subversion = useSubversion()

  // const [entries, setEntries] = useState<LogEntry[]>([])
  // const instantEntries = useRef<LogEntry[]>([])
  // const [isBottomLoading, setIsBottomLoading] = useState(false)
  // const [isTopLoading, setIsTopLoading] = useState(false)
  // const [reachBottom, setReachBottom] = useState(false)
  // const [reachTop, setReachTop] = useState(false)
  // const [selectedRevision, setSelectedRevision] = useState<number | null>(null)
  // const [repositoryRootUrl, setRepositoryRootUrl] = useState<string | null>(null)

  // const repositoryRootUrlRef = useRef<string | null>(null)
  // const urlRef = useRef(props.url)
  // const editor = useRef<StrongDifferenceEditorRef | null>(null)

  // const sortEntries = (list: LogEntry[]) =>
  //   [...list].sort((a, b) => (b.revision ?? 0) - (a.revision ?? 0))

  // // changed path 的键是相对仓库根目录的路径（如 /trunk/file.txt），
  // // 用 url 减去仓库根 URL 得到相对路径来查找该文件在某个修订版的改动信息
  // const changedEntryOf = useMemoizedFn((entry: LogEntry): LogChangedPathEntry | undefined => {
  //   const root = repositoryRootUrlRef.current
  //   if (root === null) {
  //     return undefined
  //   }
  //   return entry.changedPathEntries[trimStartSubstring(props.url, root)]
  // })

  // // 向更旧的修订版分页获取（每次最多 limit 条，滚动到底部时继续）
  // const loadBottom = useMemoizedFn(async () => {
  //   if (reachBottom || isBottomLoading) {
  //     return
  //   }
  //   setIsBottomLoading(true)
  //   try {
  //     await Subversion.callOnce({
  //       factory: subversion,
  //       call: async (context) => {
  //         const current = instantEntries.current
  //         const minRevision = current.length > 0 ? current[current.length - 1].revision : null

  //         const options: LogOptions = {
  //           targets: [props.url],
  //           pegRevision: props.pegRevision,
  //           limit,
  //           revisions: [
  //             {
  //               start: minRevision !== null ? { number: minRevision } : props.pegRevision,
  //               end: { number: 0 },
  //             },
  //           ],
  //           discoverChangedPaths: true,
  //           strictNodeHistory: false,
  //           includeMergedRevisions: false,
  //           revisionsProperties: null,
  //         }

  //         let fetched = (await context.log(options)).entries
  //         if (minRevision !== null) {
  //           fetched = fetched.filter((e) => e.revision !== minRevision)
  //         }
  //         if (fetched.length < limit) {
  //           setReachBottom(true)
  //         }
  //         instantEntries.current = sortEntries([...instantEntries.current, ...fetched])
  //         setEntries(instantEntries.current)
  //       },
  //       onError: (error) => {
  //         Toast.error({
  //           content: `Failed to load log: ${error}`,
  //           stack: true,
  //         })
  //       },
  //     })
  //   } finally {
  //     setIsBottomLoading(false)
  //   }
  // })

  // // 向更新的修订版分页获取（滚动到顶部时继续，直到 head）
  // const loadTop = useMemoizedFn(async () => {
  //   if (isTopLoading || reachTop) {
  //     return
  //   }
  //   const current = instantEntries.current
  //   if (current.length === 0) {
  //     return
  //   }
  //   const maxRevision = current[0].revision
  //   if (maxRevision === null) {
  //     return
  //   }

  //   setIsTopLoading(true)
  //   try {
  //     await Subversion.callOnce({
  //       factory: subversion,
  //       call: async (context) => {
  //         let start = maxRevision
  //         while (true) {
  //           const options: LogOptions = {
  //             targets: [props.url],
  //             pegRevision: props.pegRevision,
  //             limit,
  //             revisions: [{ start: { number: start }, end: 'head' }],
  //             discoverChangedPaths: true,
  //             strictNodeHistory: false,
  //             includeMergedRevisions: false,
  //             revisionsProperties: null,
  //           }

  //           const fetched = (await context.log(options)).entries.filter(
  //             (e) => e.revision !== start,
  //           )
  //           if (fetched.length === 0) {
  //             setReachTop(true)
  //             break
  //           }

  //           instantEntries.current = sortEntries([...instantEntries.current, ...fetched])
  //           setEntries(instantEntries.current)

  //           start = fetched.reduce((max, e) => Math.max(max, e.revision ?? 0), start)
  //           if (fetched.length < limit) {
  //             setReachTop(true)
  //             break
  //           }
  //         }
  //       },
  //       onError: (error) => {
  //         Toast.error({
  //           content: `Failed to load log: ${error}`,
  //           stack: true,
  //         })
  //       },
  //     })
  //   } finally {
  //     setIsTopLoading(false)
  //   }
  // })

  // const pegKey = JSON.stringify(props.pegRevision)

  // // url / peg revision 变化时重置并重新加载
  // useEffect(() => {
  //   const url = props.url

  //   instantEntries.current = []
  //   setEntries([])
  //   setReachBottom(false)
  //   setReachTop(false)
  //   setSelectedRevision(null)
  //   editor.current?.clear()
  //   urlRef.current = url

  //   const call = async () => {
  //     // 先取仓库根 URL（用于解析 changed path 键），再开始分页加载日志
  //     await Subversion.callOnce({
  //       factory: subversion,
  //       call: async (context) => {
  //         const options: InfoOptions = {
  //           path: url,
  //           pegRevision: props.pegRevision,
  //           revision: 'unspecified',
  //           depth: 'empty',
  //           fetchExcluded: false,
  //           fetchActualOnly: false,
  //           includeExternals: false,
  //           changelists: null,
  //         }
  //         const result = await context.info(options)
  //         if (urlRef.current !== url) {
  //           return
  //         }
  //         const infos = Object.values(result.entries)
  //         if (infos.length !== 1) {
  //           throw new Error(`Expected exactly one info entry for ${url}`)
  //         }
  //         repositoryRootUrlRef.current = infos[0].repositoryRootUrl
  //         setRepositoryRootUrl(infos[0].repositoryRootUrl)
  //       },
  //       onError: (error) => {
  //         Toast.error({
  //           content: `Failed to query info of ${url}: ${error}`,
  //           stack: true,
  //         })
  //       },
  //     })
  //     if (urlRef.current !== url) {
  //       return
  //     }
  //     await loadBottom()
  //   }
  //   call()
  //   // eslint-disable-next-line react-hooks/exhaustive-deps
  // }, [props.url, pegKey])

  // // 构造某个修订版与上一个修改该文件的修订版之间的对比目标
  // const createTarget = useMemoizedFn(
  //   (entry: LogEntry, knownOlderRevision: number | null): IDifferenceTarget => {
  //     const revision = entry.revision
  //     const action = changedEntryOf(entry)?.action
  //     // undefined 表示"上一个修订版未知"（选中的是当前已加载的最旧条目），
  //     // 第一次需要旧内容时再用一次 log 查询（limit 1）按需解析
  //     let compareRevision: number | null | undefined = knownOlderRevision

  //     const resolveCompareRevision = async (): Promise<number | null> => {
  //       if (compareRevision !== undefined) {
  //         return compareRevision
  //       }
  //       if (revision === null) {
  //         compareRevision = null
  //         return null
  //       }
  //       const result = await Subversion.callOnce({
  //         factory: subversion,
  //         call: async (context) => {
  //           const options: LogOptions = {
  //             targets: [props.url],
  //             pegRevision: props.pegRevision,
  //             limit: 1,
  //             revisions: [{ start: { number: revision - 1 }, end: { number: 0 } }],
  //             discoverChangedPaths: false,
  //             strictNodeHistory: false,
  //             includeMergedRevisions: false,
  //             revisionsProperties: null,
  //           }
  //           const fetched = (await context.log(options)).entries
  //           return fetched.length > 0 ? fetched[0].revision : null
  //         },
  //       })
  //       compareRevision = result ?? null
  //       return compareRevision
  //     }

  //     const loadContentAt = async (
  //       rev: number,
  //       expandKeywords: boolean,
  //     ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> => {
  //       const options: CatOptions = {
  //         path: props.url,
  //         pegRevision: { number: rev },
  //         revision: { number: rev },
  //         expandKeywords,
  //         getProperties: false,
  //       }
  //       return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewContent']>>>({
  //         factory: subversion,
  //         call: async (context) => {
  //           const content = (await context.cat(options)).content
  //           const decoder = new TextDecoder('utf-8', { fatal: true })
  //           try {
  //             return { text: decoder.decode(content) }
  //           } catch {
  //             // 不是合法 UTF-8，按二进制文件展示
  //             const name = props.url.substring(props.url.lastIndexOf('/') + 1)
  //             return { file: { name, size: content.length } }
  //           }
  //         },
  //         onError: (error) => {
  //           const subversionError = getSubversionError(error)
  //           if (subversionError && subversionError.code === 'clientIsDirectory') {
  //             return null
  //           }
  //           return { error }
  //         },
  //       })
  //     }

  //     const loadPropertyAt = async (
  //       rev: number,
  //     ): Promise<{ property: PropertyListEntry } | { error: any } | null> => {
  //       return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewProperty']>>>({
  //         factory: subversion,
  //         call: async (context) => {
  //           const options: PropertyListOptions = {
  //             target: props.url,
  //             pegRevision: { number: rev },
  //             revision: { number: rev },
  //             depth: 'empty',
  //             changelists: null,
  //             inherited: false,
  //           }
  //           const result = await context.propertyList(options)
  //           return {
  //             property: result.entries[0],
  //           }
  //         },
  //         onError: (error) => {
  //           return { error }
  //         },
  //       })
  //     }

  //     return {
  //       loadNewContent: async (expandKeywords) => {
  //         if (revision === null || action === 'delete') {
  //           return null
  //         }
  //         return await loadContentAt(revision, expandKeywords)
  //       },
  //       loadOldContent: async (expandKeywords) => {
  //         if (revision === null || action === 'add') {
  //           return null
  //         }
  //         const compare = await resolveCompareRevision()
  //         if (compare === null) {
  //           return null
  //         }
  //         return await loadContentAt(compare, expandKeywords)
  //       },
  //       loadNewProperty: async () => {
  //         if (revision === null || action === 'delete') {
  //           return null
  //         }
  //         return await loadPropertyAt(revision)
  //       },
  //       loadOldProperty: async () => {
  //         if (revision === null || action === 'add') {
  //           return null
  //         }
  //         const compare = await resolveCompareRevision()
  //         if (compare === null) {
  //           return null
  //         }
  //         return await loadPropertyAt(compare)
  //       },
  //     }
  //   },
  // )

  // // 选中修订版变化时向差异编辑器注册对比目标
  // useEffect(() => {
  //   if (!props.showDifference) {
  //     return
  //   }
  //   if (selectedRevision === null) {
  //     return
  //   }
  //   const current = instantEntries.current
  //   const index = current.findIndex((e) => e.revision === selectedRevision)
  //   if (index < 0) {
  //     return
  //   }
  //   const knownOlder = index + 1 < current.length ? current[index + 1].revision : null
  //   editor.current?.add(String(selectedRevision), createTarget(current[index], knownOlder))
  //   // eslint-disable-next-line react-hooks/exhaustive-deps
  // }, [selectedRevision, repositoryRootUrl, props.showDifference, props.url, pegKey])

  const [entries, _setEntries] = useState<LogEntry[]>([])
  const editor = useRef<StrongDifferenceEditorRef>(null)
  const [selectedRevision, _setSelectedRevision] = useState<number | null>(null)

  return (
    <div
      className={cx(
        flex,
        flex_col,
        gap_y_1,
        flex_1,
        min_h_0,
        '__________filehistory',
        props.className,
      )}
    >
      <Group orientation="horizontal" className={cx(flex_1, min_h_0)}>
        <Panel className={cx(flex)} defaultSize={'40%'}>
          <VirtualList
            count={entries.length}
            itemRender={function (_: VirtualItem) {
              return <div></div>
            }}
            itemHeight={0}
          ></VirtualList>
        </Panel>
        <Separator style={{ width: 4 }} />
        <Panel className={cx(flex)}>
          <StrongDifferenceEditor
            ref={editor}
            currentKey={selectedRevision === null ? undefined : String(selectedRevision)}
            className={cx(flex_1)}
          />
        </Panel>
      </Group>
    </div>
  )
}
