'use no memo'

import { Spin } from '@douyinfe/semi-ui'
import RefreshIcon from '@icons/Refresh.svg?react'
import { cx } from '@linaria/core'
import { useCallback, useEffect, useState } from 'react'
import { createPortal } from 'react-dom'

import { InfoOptions } from '@/bindings/InfoOptions'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import { Subversion, useSubversion } from '@/context/Subversion'
import {
  flex,
  flex_1,
  flex_col,
  gap_x_1,
  items_center,
  min_h_0,
  min_w_0,
  overflow_hidden,
  text_12px,
  whitespace_nowrap,
} from '@/styles/Classes'
import { trimStartSubstring } from '@/utils/String'

import { SnapshotView } from '../HistoryView/SnapshotView'
import { useWorkingCopyContext } from '../WorkingCopyView'

export interface RemoteViewProps {
  className?: string
}

interface RemoteTarget {
  url: string
  root: string
  location: string
  revision: number
}

export function RemoteView(props: RemoteViewProps) {
  const workingCopy = useWorkingCopyContext()
  const subversion = useSubversion()
  const [target, setTarget] = useState<RemoteTarget | null>(null)
  const [isLoading, setIsLoading] = useState(true)
  const [reload, setReload] = useState(0)

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
          throw new Error(`${workingCopy.path} is not a working copy of a repository`)
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

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: 'Refresh',
      enable: !isLoading,
      children: <RefreshIcon></RefreshIcon>,
      onClick: async () => {
        setReload((value) => value + 1)
      },
    },
  ]

  const bar = (
    <div className={cx(flex, items_center, gap_x_1, overflow_hidden)}>
      <OperationBar size={25} icons={icons}></OperationBar>
      {target !== null && (
        <span
          className={cx(text_12px, whitespace_nowrap, overflow_hidden)}
          style={{ color: 'var(--semi-color-text-2)' }}
          title={`${target.url} @ r${target.revision}`}
        >
          {target.url} @ r{target.revision}
        </span>
      )}
    </div>
  )

  return (
    <div
      className={cx(props.className, flex, flex_col, min_w_0, min_h_0, overflow_hidden)}
    >
      {workingCopy.remoteViewOperationContainer !== null &&
        createPortal(bar, workingCopy.remoteViewOperationContainer)}
      <Spin
        spinning={isLoading}
        wrapperClassName={cx(flex_1, flex, min_w_0, min_h_0)}
        childStyle={{ display: 'flex', minWidth: 0, minHeight: 0, flex: 1 }}
      >
        {target === null ? (
          <></>
        ) : (
          <SnapshotView
            key={`${target.url}_${target.revision}`}
            className={cx(flex_1, min_w_0, min_h_0)}
            root={target.root}
            location={target.location}
            pegRevision={target.revision}
            revision={target.revision}
          ></SnapshotView>
        )}
      </Spin>
    </div>
  )
}
