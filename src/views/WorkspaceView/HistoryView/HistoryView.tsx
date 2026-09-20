'use no memo'
import { Spin, TabPane, Tabs, Toast } from '@douyinfe/semi-ui'
import LoadMoreIcon from '@icons/LoadMore.svg?react'
import SearchIcon from '@icons/Search.svg?react'
import { css, cx } from '@linaria/core'
import type { ColumnDef } from '@tanstack/react-table'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { useMemoizedFn, useReactive } from 'ahooks'
import { useEffect, useMemo, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { Group, Separator, Panel } from 'react-resizable-panels'
import * as uuid from 'uuid'

import { InfoEntry } from '@/bindings/InfoEntry'
import { InfoOptions } from '@/bindings/InfoOptions'
import { LogEntry } from '@/bindings/LogEntry'
import { LogOptions } from '@/bindings/LogOptions'
import { RevisionPropertyName } from '@/bindings/RevisionPropertyName'
import LazyComponent from '@/components/LazyComponent'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import PureInput from '@/components/PureInput'
import { Table, TableSelection } from '@/components/Table'
import { databaseRevisionLocation, databaseUpdateRevisionLocation } from '@/context/Functions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useModal } from '@/lib/multi-modal'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  gap_x_1,
  gap_y_1,
  hidden,
  items_center,
  min_h_0,
  min_w_0,
  overflow_hidden,
} from '@/styles/Classes'
import { MessagePackChannel } from '@/utils/MessagePack'
import { SingleTaskQueue } from '@/utils/Queue'
import { trimStartSubstring } from '@/utils/String'
import { NiceMergeDialog } from '@/views/dialogs/MergeDialog'

import { useWorkingCopyContext } from '../WorkingCopyView'
import { ChangesView } from './ChangesView'
import DetailView from './DetailView'
import { SnapshotView } from './SnapshotView'

const shadow = css`
  box-shadow: var(--semi-shadow-elevated);
  border-radius: 5px;
  /*background-color: rgba(var(--semi-grey-0), 1);*/
  background-color: var(--semi-color-bg-0);
`

interface RevisionLogViewProps {
  className?: string
  onSelectionChanged?: (entry: LogEntry | null) => void
  onReady?: (entry: InfoEntry) => void
}

function RevisionLogView(props: RevisionLogViewProps) {
  const maxLogSize = 100
  type Data = {
    revision?: number
    message?: string
    author?: string
    date?: string
    uuid?: string
    entry: LogEntry
  }
  const state = useReactive({
    isBottomLoading: false,
    isTopLoading: false,
  })
  // const [isBottomLoading, setIsBottomLoading] = useState(false)
  // const [isTopLoading, setIsTopLoading] = useState(false)
  const [entries, setEntries] = useState<LogEntry[]>([])
  const info = useRef<InfoEntry | null>(null)
  const [reachBottom, setReachBottom] = useState(false)
  const instantEntries = useRef<LogEntry[]>([])

  // Column Definitions: Defines the columns to be displayed.
  const columnDefinitions = useMemo<ColumnDef<Data>[]>(() => {
    return [
      { accessorKey: 'message', header: 'Message', size: 400 },
      { accessorKey: 'author', header: 'Author', size: 100 },
      { accessorKey: 'revision', header: 'Revision', size: 100 },
      { accessorKey: 'date', header: 'Date', size: 200 },
    ]
  }, [])

  const workingCopy = useWorkingCopyContext()
  const subversion = useSubversion()

  const loadBottom = async () => {
    if (reachBottom || state.isBottomLoading) {
      return
    }
    state.isBottomLoading = true
    // setIsBottomLoading(true)
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        if (info.current === null) {
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
          const result = await context.info(options)
          const entries = Object.entries(result.entries)
          if (entries.length !== 1) {
            Toast.error({
              content: `Failed to query repository of ${workingCopy.path}`,
              stack: true,
            })
            return
          }

          info.current = entries[0][1]
          props.onReady?.(info.current)
        }

        if (info.current.revision === null || info.current.url === null) {
          return
        }

        const limit = 100
        let start: number | undefined = undefined

        if (instantEntries.current.length > 0) {
          const revision = instantEntries.current[instantEntries.current.length - 1].revision
          if (revision !== null) {
            start = revision
          }
        }

        const options: LogOptions = {
          targets: [info.current.url],
          pegRevision: { number: info.current.revision },
          limit,
          revisions: [
            { start: start !== undefined ? { number: start } : 'head', end: { number: 0 } },
          ],
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
            if (start !== undefined) {
              if (entry.revision === start) {
                return
              }
            }
            if (instantEntries.current.findIndex((i) => i.revision === entry.revision) >= 0) {
              console.warn('Log has been loaded:', entry.revision, options, uuid)
            }
            count++
            instantEntries.current = [...instantEntries.current, entry]

            setEntries(instantEntries.current)
          }
        })


        await context.logNext(options, channel)

        await promise


        const offset = instantEntries.current.length - maxLogSize

        if (offset > 0 && !state.isTopLoading) {
          instantEntries.current = instantEntries.current.slice(offset)
          setEntries(instantEntries.current)
        }

        setReachBottom(count === 0)

        // if (count < limit) {
        //   console.log('set reach bottom: ', count, limit)
        //   setReachBottom(true)
        // }

        // if (start) {
        //   logEntries = logEntries.filter((e) => e.revision !== start)
        // }

        // if (logEntries.length < limit) {
        //   setReachBottom(true)
        // }
        // console.log('got entry', logEntries)

        // instantEntries.current = [...instantEntries.current, ...logEntries]

        // setEntries(instantEntries.current)

        // setEntries((entries) => [...entries, ...logEntries])
      },
    })
    state.isBottomLoading = false
    console.log('Finished loading bottom', uuid)
    // setIsBottomLoading(false)
  }

  const loadTop = async () => {
    if (instantEntries.current.length === 0) {
      return
    }
    if (instantEntries.current[0].revision === null) {
      return
    }

    if (info.current === null) {
      return
    }

    const infoEntry = info.current

    if (infoEntry.url === null) {
      return
    }

    const url = infoEntry.url

    if (state.isTopLoading) {
      return
    }

    // setIsTopLoading(true)
    state.isTopLoading = true

    let start = instantEntries.current[0].revision

    await Subversion.callOnce({
      factory: subversion,
      call: async (context) => {
        while (true) {
          const options: LogOptions = {
            targets: [url],
            pegRevision: { number: infoEntry.revision! },
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
              console.log('Load top add entry: ', entry)
              instantEntries.current = [entry, ...instantEntries.current]
              setEntries(instantEntries.current)
            }
          })
          console.log('Ready to load top', options)
          await context.logNext(options, channel)
          await promise
          console.log('load top finished', options)

          const offset = instantEntries.current.length - maxLogSize
          if (offset > 0 && !state.isBottomLoading) {
            console.log('too many logs, remove now', instantEntries.current)
            instantEntries.current = instantEntries.current.slice(0, -offset)
            console.log('too many logs, remove', instantEntries.current)
            setEntries(instantEntries.current)
            setReachBottom(false)
          }

          // instantEntries.current = [
          //   ...result.filter((e) => e.revision !== start),
          //   ...instantEntries.current,
          // ]
          // setEntries(instantEntries.current)
          // setEntries(items => [...result, ...items])
          // if (result.length > 0) {
          //   start = result[0].revision!
          // }
          start = instantEntries.current[0].revision!
          if (count < 100) {
            break
          }
        }
      },
    })

    state.isTopLoading = false
    // setIsTopLoading(false)
  }

  useEffect(() => {
    const call = async () => {
      await loadBottom()
      await loadTop()
    }
    call()
  }, [])

  const rowData: Data[] = useMemo(() => {
    return entries.map((e) => {
      function revisionsValue(key: RevisionPropertyName): string | undefined {
        if (e.revisionProperties === null) {
          return undefined
        }
        return e.revisionProperties[key]
      }

      return {
        revision: e.revision === null ? undefined : e.revision,
        message: revisionsValue('svn:log'),
        author: revisionsValue('svn:author'),
        date: revisionsValue('svn:date'),
        uuid: e.revision === undefined ? uuid.v4() : undefined,
        entry: e,
      }
    })
  }, [entries])
  console.log('rowData', rowData)

  // const onBodyScroll: UIEventHandler<HTMLDivElement> = (event) => {
  //   console.log('on body scroll: ', event)

  //   const element = event.currentTarget

  //   if (element.scrollHeight - element.scrollTop - element.clientHeight < 200) {
  //     if (!isBottomLoading && !reachBottom) {
  //       loadBottom()
  //     }
  //   }

  //   if (!isTopLoading) {
  //     if (element.scrollTop < 200) {
  //       loadTop()
  //     }
  //   }
  // }

  const onSelectionChanged = (selection: TableSelection | null) => {
    if (selection === null) {
      props.onSelectionChanged?.(null)
      return
    }

    if (selection.type === 'row') {
      if (selection.rowIds.length === 1) {
        const revision = Number(selection.rowIds[0])
        const entry = entries.find((e) => e.revision == revision)
        if (entry) {
          props.onSelectionChanged?.(entry)
        }
      }
    }
  }

  const [enableSearch, setEnableSearch] = useState(false)
  const searchInput = useRef<HTMLInputElement | null>(null)
  const focusSearchInput = useMemoizedFn(() => {
    if (enableSearch) {
      searchInput.current?.focus()
    }
  })

  const icons: OperationIconProps[] = [
    {
      tooltip: 'Load more',
      sync: false,
      enable: true,
      children: <LoadMoreIcon></LoadMoreIcon>,
      onClick: async () => {
        await loadBottom()
        await loadTop()
      },
    },
    {
      tooltip: 'Search',
      enable: true,
      children: (
        <SearchIcon
          style={{ color: enableSearch ? 'var(--semi-color-primary)' : undefined }}
        ></SearchIcon>
      ),
      onClick: () => {
        setEnableSearch((v) => !v)
        if (enableSearch) {
          // old value
          setSearchText('')
        }
        setTimeout(() => {
          focusSearchInput()
        }, 0)
      },
    },
  ]

  const bar = (
    <div className={cx(flex, items_center)}>
      <OperationBar className={cx(gap_x_1, flex_1)} size={25} icons={icons}></OperationBar>
    </div>
  )

  const [searchText, setSearchText] = useState('')
  const modal = useModal()

  return (
    <div className={cx(flex, flex_col, gap_y_1, props.className)}>
      <PureInput
        ref={searchInput}
        className={cx(!enableSearch && hidden)}
        autoFocus
        value={searchText}
        onChange={setSearchText}
      ></PureInput>
      <div className={cx(flex, flex_1, overflow_hidden, shadow)}>
        {workingCopy.historyViewOperationContainer !== null &&
          createPortal(bar, workingCopy.historyViewOperationContainer)}
        <Table
          selectionMode="row"
          data={rowData}
          onBottomReached={loadBottom}
          onTopReached={loadTop}
          columns={columnDefinitions}
          className={cx(flex_1, min_h_0, min_w_0)}
          loading={state.isBottomLoading || state.isTopLoading}
          threshold={1}
          onSelectionChange={onSelectionChanged}
          onGetRowId={(e) => (e.revision === undefined ? (e.uuid ?? '') : String(e.revision))}
          globalFilter={searchText}
          rowContextMenu={(row) => {
            return [
              {
                item: {
                  content: 'Copy message',
                  onSelect: () => {
                    if (row.original.message) {
                      writeText(row.original.message)
                    }
                  },
                },
              },
              {
                item: {
                  content: 'Copy author',
                  onSelect: () => {
                    if (row.original.author) {
                      writeText(row.original.author)
                    }
                  },
                },
              },
              {
                item: {
                  content: 'Copy revision',
                  onSelect: () => {
                    if (row.original.revision) {
                      writeText(String(row.original.revision))
                    }
                  },
                },
              },
              {
                item: {
                  content: 'Copy date',
                  onSelect: () => {
                    if (row.original.date) {
                      writeText(row.original.date)
                    }
                  },
                },
              },
              {
                item: {
                  content: 'Merge(reset)',
                  onSelect: () => {
                    if (row.original.revision === undefined) {
                      return
                    }
                    modal.show(NiceMergeDialog, {
                      defaultTarget: workingCopy.path,
                      defaultSource: {
                        peg: {
                          source: workingCopy.path,
                          pegRevision: 'base',
                          rangesToMerge: [
                            {
                              start: 'head',
                              end: {
                                number: row.original.revision,
                              },
                            },
                          ],
                        },
                      },
                    })
                  },
                },
              },
            ]
          }}
        />
      </div>
    </div>
  )
}

export interface HistoryViewProps {
  className?: string
}

const contentPadding = css`
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  padding-top: 0px;
`

export function HistoryView(props: HistoryViewProps) {
  const detailKey = 'detailKey'
  const changesKey = 'changesKey'
  const snapshotKey = 'snapshotKey'
  const [activeView, setActiveView] = useState(detailKey)
  const [selectedlogEntry, setSelectedLogEntry] = useState<LogEntry | null>(null)
  const [location, setLocation] = useState<string | null>(null)
  const subversion = useSubversion()
  const [isLoading, setIsLoading] = useState(false)
  const queue = useRef(new SingleTaskQueue())
  // const locations = useRef<LimitedDictionary<number, String>>(new LimitedDictionary(300))
  const [infoEntry, setInfoEntry] = useState<InfoEntry | null>(null)

  // const workingCopy = useWorkingCopyContext()

  useEffect(() => {
    queue.current.onQueueEmpty = () => {
      setIsLoading(false)
    }
  }, [])

  useEffect(() => {
    const call = async (signal: AbortSignal) => {
      if (
        infoEntry === null ||
        infoEntry.revision === null ||
        infoEntry.url === null ||
        infoEntry.repositoryRootUrl === null ||
        infoEntry.repositoryUuid === null
      ) {
        return
      }
      if (selectedlogEntry && selectedlogEntry.revision) {
        if (signal.aborted) return
        setIsLoading(true)
        try {
          const path = trimStartSubstring(infoEntry.url, infoEntry.repositoryRootUrl)

          const url = infoEntry.url
          const repository = infoEntry.repositoryUuid
          const pegRevision = infoEntry.revision
          const revision = selectedlogEntry.revision

          const result = await databaseRevisionLocation(repository, path, pegRevision, revision)
          console.log('On revision location get result:', result, infoEntry)
          if (signal.aborted) return
          if (result === null) {
            await Subversion.callOnce({
              factory: subversion,
              call: async (context) => {
                if (signal.aborted) return
                const result = await context.raGetLocations(url, pegRevision, [revision])
                let location = result[revision]
                await databaseUpdateRevisionLocation(
                  repository,
                  path,
                  pegRevision,
                  revision,
                  location,
                )
                if (signal.aborted) return
                setLocation(location)
              },
            })
          } else {
            setLocation(result)
          }
        } finally {
        }
      }
    }
    // call()
    queue.current.run(call)
  }, [selectedlogEntry])

  return (
    <div style={{ marginTop: '5px' }} className={cx(flex, border_box, props.className)}>
      <Group className={cx(flex_1)} orientation="vertical">
        <Panel className={cx(flex)} defaultSize={'60%'}>
          <RevisionLogView
            onReady={setInfoEntry}
            onSelectionChanged={setSelectedLogEntry}
            className={cx(flex_1, min_h_0, min_w_0)}
          ></RevisionLogView>
        </Panel>
        <Separator style={{ height: 3 }} />
        <Panel className={cx(flex)}>
          <div
            className={cx(
              flex,
              flex_col,
              flex_1,
              shadow,
              border_box,
              min_h_0,
              contentPadding,
              min_w_0,
            )}
          >
            <Tabs
              size="small"
              defaultActiveKey={detailKey}
              contentStyle={{ display: 'none' }}
              tabPaneMotion={false}
              activeKey={activeView}
              onChange={(e) => setActiveView(e)}
            >
              <TabPane tab="Detail" itemKey={detailKey}></TabPane>
              <TabPane tab="Chagnes" itemKey={changesKey}></TabPane>
              <TabPane tab="Snapshot" itemKey={snapshotKey}></TabPane>
            </Tabs>
            <Spin
              childStyle={{ flex: 1, display: 'flex', minHeight: 0, minWidth: 0 }}
              wrapperClassName={cx(flex, flex_1, min_h_0, min_w_0)}
              spinning={isLoading}
            >
              <div className={cx(flex_1, flex, min_h_0, min_w_0)}>
                {selectedlogEntry === null || location === null || infoEntry === null ? (
                  <></>
                ) : (
                  <LazyComponent
                    key={`${detailKey}_${selectedlogEntry.revision ?? 0}`}
                    visible={activeView === detailKey}
                  >
                    <DetailView
                      root={infoEntry.repositoryRootUrl ?? ''}
                      location={location}
                      relateTo={location}
                      entry={selectedlogEntry}
                      className={cx(activeView !== detailKey && hidden, flex_1)}
                    ></DetailView>
                  </LazyComponent>
                )}
                {selectedlogEntry === null || location === null ? (
                  <></>
                ) : (
                  <LazyComponent
                    key={`${changesKey}_${selectedlogEntry.revision ?? 0}`}
                    visible={activeView === changesKey}
                  >
                    <ChangesView
                      repositoryRoot={infoEntry?.repositoryRootUrl ?? ''}
                      currentRevision={selectedlogEntry.revision ?? 0}
                      compareRevision={Math.max((selectedlogEntry.revision ?? 0) - 1, 0)}
                      relativePath={location}
                      entries={selectedlogEntry.changedPathEntries}
                      className={cx(activeView !== changesKey && hidden, flex_1)}
                    ></ChangesView>
                  </LazyComponent>
                )}
                {selectedlogEntry === null ||
                selectedlogEntry.revision === null ||
                location == null ||
                infoEntry === null ||
                infoEntry.repositoryRootUrl === null ? (
                  <></>
                ) : (
                  <LazyComponent
                    key={`${snapshotKey}_${selectedlogEntry.revision}`}
                    visible={activeView === snapshotKey}
                  >
                    <SnapshotView
                      className={cx(activeView !== snapshotKey && hidden, flex_1)}
                      location={location}
                      root={infoEntry.repositoryRootUrl}
                      // url={combineUrl(infoEntry.repositoryRootUrl, location)}
                      pegRevision={selectedlogEntry.revision}
                      revision={selectedlogEntry.revision}
                    />
                  </LazyComponent>
                )}
              </div>
            </Spin>
          </div>
        </Panel>
      </Group>
    </div>
  )
}
