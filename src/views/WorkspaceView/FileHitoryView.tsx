import ChangeActionIcon from '@components/subversion/ChangeActionIcon'
import { Spin, Typography } from '@douyinfe/semi-ui'
import UserAvatarIcon from '@icons/UserAvatar.svg?react'
import { css, cx } from '@linaria/core'
import { useMemoizedFn, useReactive } from 'ahooks'
import dayjs from 'dayjs'
import { useEffect, useMemo, useRef, useState } from 'react'
import { Group, Panel, Separator } from 'react-resizable-panels'
import { useParams } from 'react-router'

import { CatOptions } from '@/bindings/CatOptions'
import { InfoEntry } from '@/bindings/InfoEntry'
import { InfoOptions } from '@/bindings/InfoOptions'
import { LogEntry } from '@/bindings/LogEntry'
import { LogOptions } from '@/bindings/LogOptions'
import { PropertyListEntry } from '@/bindings/PropertyListEntry'
import { PropertyListOptions } from '@/bindings/PropertyListOptions'
import { RevisionPropertyName } from '@/bindings/RevisionPropertyName'
import { BinaryFile } from '@/components/DifferenceEditor'
import HoverTooltip from '@/components/HoverTooltip'
import LazyVirtualList from '@/components/LazyVirtualList'
import StrongDifferenceEditor, {
  IDifferenceTarget,
  StrongDifferenceEditorRef,
} from '@/components/StrongDifferenceEditor'
import { base64Decode } from '@/context/Functions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { ModalProvider } from '@/lib/multi-modal'
import {
  absolute,
  border_box,
  flex,
  flex_1,
  flex_col,
  flex_shrink_0,
  gap_x_1,
  gap_y_1,
  h_full,
  hidden,
  items_center,
  min_h_0,
  overflow_hidden,
  px_2,
  py_1,
  relative,
  w_full,
  whitespace_nowrap,
} from '@/styles/Classes'
import { list_item, list_item_selected } from '@/styles/Components'
import { TabContent } from '@/tab/Tab'
import { getSubversionError } from '@/utils/Error'
import Logger from '@/utils/Logger'
import { MessagePackChannel } from '@/utils/MessagePack'
import { repoPath } from '@/utils/Path'

import { SubversionProvider } from '../SubversionProvider'

export interface FileHistoryViewProps {
  showDifference: boolean
  url: string
  pegRevision: number
  className?: string
}

const itemLine = css`
  position: relative;

  &::after {
    content: '';
    position: absolute;
    bottom: 0;
    left: 6px;
    right: 6px;
    height: 1px;
    background: rgba(var(--semi-grey-1), 1);
  }
`

export function FileHistoryItem({
  path,
  entry,
  className,
  onClick,
}: {
  path?: string
  entry: LogEntry
  className?: string
  onClick?: () => void
}) {
  const authorKey: RevisionPropertyName = 'svn:author'
  const dateKey: RevisionPropertyName = 'svn:date'
  const logKey: RevisionPropertyName = 'svn:log'

  const author = entry.revisionProperties?.[authorKey]
  const date = entry.revisionProperties?.[dateKey]
  const log = entry.revisionProperties?.[logKey]

  const action =
    path !== undefined && path in entry.changedPathEntries
      ? entry.changedPathEntries[path].action
      : undefined

  const name = path !== undefined ? repoPath.getFileName(path) : undefined

  const prettyDate = useMemo(() => {
    if (date === undefined) {
      return undefined
    }
    return dayjs(date).format('YYYY-MM-DD')
  }, [date])

  return (
    <>
      <div
        onClick={onClick}
        className={cx(
          flex,
          flex_col,
          whitespace_nowrap,
          overflow_hidden,
          border_box,
          py_1,
          px_2,
          className,
        )}
      >
        <div
          className={cx(
            flex,
            gap_x_1,
            items_center,
            overflow_hidden,
            css`
              font-size: 14px;
            `,
          )}
        >
          <UserAvatarIcon className={cx(flex_shrink_0)} style={{ width: 20, height: 20 }} />
          <span>{author}</span>
          <div className={cx(flex_1)}></div>
          <HoverTooltip content={date} style={{ maxWidth: 300 }}>
            {prettyDate}
          </HoverTooltip>
        </div>
        <Typography.Text
          style={{ fontSize: 16 }}
          className={cx(overflow_hidden, flex_1, border_box, py_1)}
        >
          {log}
        </Typography.Text>
        <span className={cx(flex, gap_x_1, items_center, overflow_hidden)}>
          {action !== undefined && (
            <ChangeActionIcon className={cx(flex_shrink_0)} action={action} />
          )}
          {name !== undefined && (
            <Typography.Text className={cx(whitespace_nowrap)}>{name}</Typography.Text>
          )}
          <div className={cx(flex_1)}></div>
          <Typography.Text type="tertiary" size="small">{`r${entry.revision}`}</Typography.Text>
        </span>
      </div>
    </>
  )
}

export function RouteFileHistoryView() {
  const { url, revision } = useParams()

  const [decodeUrl, setDecodeUrl] = useState<string | null>(null)

  Logger.info('Url params', url, revision)

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
      <TabContent visible>
        <ModalProvider>
          <SubversionProvider className={cx(h_full, w_full)} singleton={false}>
            <FileHistoryView
              className={cx(flex_1)}
              url={decodeUrl}
              pegRevision={Number(revision)}
              showDifference
            ></FileHistoryView>
          </SubversionProvider>
        </ModalProvider>
      </TabContent>
    )
  } else {
    return <></>
  }
}

export default function FileHistoryView(props: FileHistoryViewProps) {
  const instantEntries = useRef<LogEntry[]>([])
  const [entries, setEntries] = useState<LogEntry[]>([])
  const editor = useRef<StrongDifferenceEditorRef>(null)
  const state = useReactive({
    isTopLoading: false,
    isBottomLoading: false,
  })
  const [reachBottom, setReachBottom] = useState(false)
  const [infoEntry, setInfoEntry] = useState<InfoEntry | null>(null)
  const subversion = useSubversion()
  const [paths, setPaths] = useState<Map<number, string | Promise<string | null>>>(new Map())
  const maxLogSize = 200
  const [selectedEntry, setSelectedEntry] = useState<LogEntry | null>(null)

  const loadBottom = useMemoizedFn(async () => {
    Logger.info('Load bottom', reachBottom, state)
    if (reachBottom || state.isBottomLoading) {
      return
    }
    state.isBottomLoading = true
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        let info = infoEntry

        if (info === null) {
          const options: InfoOptions = {
            path: props.url,
            pegRevision: { number: props.pegRevision },
            revision: { number: props.pegRevision },
            depth: 'empty',
            fetchExcluded: false,
            fetchActualOnly: false,
            includeExternals: false,
            changelists: null,
          }
          const result = await context.info(options)
          const entries = Object.entries(result.entries)
          if (entries.length === 1) {
            setInfoEntry(entries[0][1])
            info = entries[0][1]
          } else {
            throw new Error('Failed to query file history: no such item')
          }
        }

        const limit = 100
        let start: number | undefined = undefined

        if (entries.length > 0) {
          const revision = entries[entries.length - 1].revision
          if (revision) {
            start = revision
          }
        }

        const options: LogOptions = {
          targets: [props.url],
          pegRevision: { number: props.pegRevision },
          limit,
          revisions: [{ start: start ? { number: start } : 'head', end: { number: 0 } }],
          discoverChangedPaths: true,
          strictNodeHistory: false,
          includeMergedRevisions: false,
          revisionsProperties: null,
        }

        let count = 0
        const channel = new MessagePackChannel<LogEntry | null>()
        const promise = new Promise((resolve) => {
          channel.onmessage = (entry) => {
            if (entry === null) {
              resolve(null)
              return
            }
            if (start) {
              if (entry.revision === start) {
                return
              }
            }
            count++
            instantEntries.current = [...instantEntries.current, entry]

            setEntries(instantEntries.current)
          }
        })

        Logger.info('Log options: ', options)

        await context.logNext(options, channel)

        await promise

        const offset = instantEntries.current.length - maxLogSize

        if (offset > 0 && !state.isTopLoading) {
          instantEntries.current = instantEntries.current.slice(offset)
          setEntries(instantEntries.current)
        }

        setReachBottom(count === 0)
      },
    })
    state.isBottomLoading = false
  })

  const loadTop = useMemoizedFn(async () => {
    Logger.info('Load bottom', reachBottom, state)
    if (instantEntries.current.length === 0) {
      return
    }
    if (instantEntries.current[0].revision === null) {
      return
    }

    if (state.isTopLoading) {
      return
    }

    state.isTopLoading = true

    let start = instantEntries.current[0].revision

    await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        while (true) {
          const options: LogOptions = {
            targets: [props.url],
            pegRevision: { number: props.pegRevision },
            limit: 0,
            revisions: [{ start: { number: start }, end: 'head' }],
            discoverChangedPaths: true,
            strictNodeHistory: false,
            includeMergedRevisions: false,
            revisionsProperties: null,
          }
          const channel = new MessagePackChannel<LogEntry | null>()
          let count = 0
          const promise = new Promise((resolve) => {
            channel.onmessage = (entry) => {
              if (entry === null) {
                resolve(null)
                return
              }
              count++
              if (entry.revision === start) {
                return
              }
              instantEntries.current = [entry, ...instantEntries.current]
              setEntries(instantEntries.current)
            }
          })
          await context.logNext(options, channel)

          await promise

          const offset = instantEntries.current.length - maxLogSize
          if (offset > 0 && !state.isBottomLoading) {
            instantEntries.current = instantEntries.current.slice(0, -offset)
            setEntries(instantEntries.current)
            setReachBottom(false)
          }

          start = instantEntries.current[0].revision!
          if (count < 100) {
            break
          }
        }
      },
    })

    state.isTopLoading = false
  })

  const loadLocation = async (revision: number) => {
    const path = paths.get(revision)
    if (typeof path === 'object') {
      return await path
    } else if (path === undefined) {
      const promise = Subversion.callOnce({
        factory: subversion,
        call: async (context) => {
          const result = await context.raGetLocations(props.url, props.pegRevision, [revision])
          if (revision in result) {
            // paths.set(revision, result[revision])
            setPaths((p) => {
              Logger.info('Update locations:', result)
              return new Map(p).set(revision, result[revision])
            })
            return result[revision]
          } else {
            Logger.warn('No location for revision=', revision)
            throw new Error(`No location for revision=${revision}`)
          }
        },
        onError: (_, handle) => {
          handle()
          setPaths((p) => {
            const map = new Map(p)
            map.delete(revision)
            return map
          })
        },
      })
      setPaths((p) => {
        return new Map(p).set(revision, promise)
      })
      return await promise
    } else {
      return path
    }
  }

  const loadPath = (revision: number) => {
    const path = paths.get(revision)
    if (typeof path === 'object') {
      return undefined
    } else if (path === undefined) {
      const promise = Subversion.callOnce({
        factory: subversion,
        call: async (context) => {
          const result = await context.raGetLocations(props.url, props.pegRevision, [revision])
          if (revision in result) {
            // paths.set(revision, result[revision])
            setPaths((p) => {
              Logger.info('Update locations:', result)
              return new Map(p).set(revision, result[revision])
            })
            return result[revision]
          } else {
            Logger.warn('No location for revision=', revision)
            throw new Error(`No location for revision=${revision}`)
          }
        },
        onError: (_, handle) => {
          handle()
          setPaths((p) => {
            const map = new Map(p)
            map.delete(revision)
            return map
          })
        },
      })
      setPaths((p) => {
        return new Map(p).set(revision, promise)
      })
      return undefined
    } else {
      return path
    }
  }

  useEffect(() => {
    const call = async () => {
      await loadBottom()
      await loadTop()
    }
    call()
  }, [])

  useEffect(() => {
    if (selectedEntry === null) {
      return
    }

    const entry = selectedEntry

    const target: IDifferenceTarget = {
      loadNewContent: async (
        expandKeywords: boolean,
      ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> => {
        if (entry.revision === null) {
          Logger.warn('Revision should not be null', entry)
          return null
        }

        const location = await loadLocation(entry.revision)
        if (location === null) {
          return {
            error: 'No location for' + String(entry.revision),
          }
        }

        if (!(location in entry.changedPathEntries)) {
          return {
            error: 'Entry is not found: ' + location,
          }
        }

        const pathEntry = entry.changedPathEntries[location]

        if (pathEntry.action === 'delete') {
          return null
        }

        if (pathEntry.nodeKind !== 'file') {
          Logger.info('Current entry is not file', pathEntry)
          return null
        }
        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewContent']>>>({
          factory: subversion,
          call: async (context) => {
            if (entry.revision === null) {
              return null
            }
            const options: CatOptions = {
              path: props.url,
              pegRevision: { number: props.pegRevision },
              revision: { number: entry.revision },
              expandKeywords,
              getProperties: false,
            }

            const result = await context.cat(options)
            const decoder = new TextDecoder('utf-8', { fatal: true })

            try {
              return {
                text: decoder.decode(result.content),
              }
            } catch {
              return {
                file: {
                  name: props.url,
                  size: result.content.length,
                },
              }
            }
          },
          onError: (error) => {
            return {
              error,
            }
          },
        })
      },
      loadOldContent: async (
        expandKeywords: boolean,
      ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> => {
        // if (.nodeStatus === 'unversioned' || entry.nodeStatus === 'added') {
        //   return null
        // }

        if (entry.revision === null) {
          return null
        }
        if (entry.revision === 0) {
          return null
        }

        const revision = entry.revision

        const location = await loadLocation(entry.revision)
        if (location === null) {
          return {
            error: 'No location for' + String(entry.revision),
          }
        }

        if (!(location in entry.changedPathEntries)) {
          return {
            error: 'Entry is not found: ' + location,
          }
        }

        const pathEntry = entry.changedPathEntries[location]

        if (pathEntry.action === 'add') {
          return null
        }

        if (pathEntry.nodeKind !== 'file') {
          return null
        }

        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadOldContent']>>>({
          factory: subversion,
          call: async (context) => {
            const options: CatOptions = {
              path: props.url,
              pegRevision: { number: props.pegRevision },
              revision: { number: revision - 1 },
              expandKeywords,
              getProperties: false,
            }

            const result = await context.cat(options)
            const decoder = new TextDecoder('utf-8', { fatal: true })

            try {
              return {
                text: decoder.decode(result.content),
              }
            } catch {
              return {
                file: {
                  name: props.url,
                  size: result.content.length,
                },
              }
            }
          },
          onError: (error) => {
            const subversion = getSubversionError(error)
            if (subversion !== undefined && subversion.code === 'clientIsDirectory') {
              return null
            }
            return {
              error,
            }
          },
        })
      },
      loadNewProperty: async (): Promise<
        { property: PropertyListEntry } | { error: any } | null
      > => {
        if (entry.revision === null) {
          return null
        }

        const revision = entry.revision

        const location = await loadLocation(entry.revision)
        if (location === null) {
          return {
            error: 'No location for' + String(entry.revision),
          }
        }

        if (!(location in entry.changedPathEntries)) {
          return {
            error: 'Entry is not found: ' + location,
          }
        }

        const pathEntry = entry.changedPathEntries[location]

        if (pathEntry.action === 'delete') {
          return null
        }

        return Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewProperty']>>>({
          factory: subversion,
          call: async (context) => {
            const options: PropertyListOptions = {
              target: props.url,
              pegRevision: { number: props.pegRevision },
              revision: { number: revision },
              depth: 'empty',
              changelists: null,
              inherited: false,
            }

            const result = await context.propertyList(options)
            return {
              property: result.entries[0],
            }
          },
          onError: (error) => {
            return {
              error,
            }
          },
        })
      },
      loadOldProperty: async (): Promise<
        { property: PropertyListEntry } | { error: any } | null
      > => {
        if (entry.revision === null) {
          return null
        }

        const revision = entry.revision

        if (revision === 0) {
          return null
        }

        const location = await loadLocation(entry.revision)
        if (location === null) {
          return {
            error: 'No location for' + String(entry.revision),
          }
        }

        if (!(location in entry.changedPathEntries)) {
          return {
            error: 'Entry is not found: ' + location,
          }
        }

        const pathEntry = entry.changedPathEntries[location]

        if (pathEntry.action === 'delete') {
          return null
        }

        return Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadOldProperty']>>>({
          factory: subversion,
          call: async (context) => {
            const options: PropertyListOptions = {
              target: props.url,
              pegRevision: { number: props.pegRevision },
              revision: { number: revision },
              depth: 'empty',
              changelists: null,
              inherited: false,
            }

            const result = await context.propertyList(options)
            return {
              property: result.entries[0],
            }
          },
          onError: (error) => {
            return {
              error,
            }
          },
        })
      },
    }

    const key = entry.revision?.toString() ?? ''
    editor.current?.add(key, target)
    //

    Logger.info('Selected entry: ', entry.revision)
  }, [selectedEntry])

  const selectedRevision = selectedEntry?.revision

  return (
    <div className={cx(flex, flex_col, gap_y_1, flex_1, min_h_0, props.className)}>
      <Group orientation="horizontal" className={cx(flex_1, min_h_0)}>
        <Panel className={cx(flex, relative)} defaultSize={'40%'}>
          <LazyVirtualList
            onLoadMoreTop={loadTop}
            onLoadMoreBottom={loadBottom}
            renderDeps={[paths, selectedRevision]}
            className={cx(flex_1)}
            count={entries.length}
            itemRender={(row, loaded) => {
              const entry = entries[row.index]
              const path = entry.revision && loaded ? loadPath(entry.revision) : undefined
              const isSelected = selectedRevision === entry.revision
              const showLine =
                row.index < entries.length - 1 &&
                !isSelected &&
                entries[row.index + 1].revision !== selectedRevision
              return (
                <FileHistoryItem
                  onClick={() => setSelectedEntry(entry)}
                  className={cx(
                    css`
                      height: 80px;
                    `,
                    showLine && itemLine,
                    list_item,
                    isSelected && list_item_selected,
                  )}
                  path={path}
                  entry={entry}
                ></FileHistoryItem>
              )
            }}
            itemHeight={80}
          ></LazyVirtualList>
          <Spin
            wrapperClassName={cx(
              css`
                bottom: 10px;
                right: 10px;
              `,
              absolute,
              !(state.isBottomLoading || state.isTopLoading) && hidden,
            )}
            spinning={state.isBottomLoading || state.isTopLoading}
          ></Spin>
        </Panel>
        <Separator style={{ width: 4 }} />
        <Panel className={cx(flex)}>
          <StrongDifferenceEditor
            ref={editor}
            currentKey={selectedEntry?.revision?.toString()}
            className={cx(flex_1)}
          />
        </Panel>
      </Group>
    </div>
  )
}
