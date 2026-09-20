import { Descriptions, Divider, Spin, Typography } from '@douyinfe/semi-ui'
import { Data } from '@douyinfe/semi-ui/lib/es/descriptions'
import { css, cx } from '@linaria/core'
import { getVersion, getTauriVersion } from '@tauri-apps/api/app'
import { useEffect, useMemo, useState } from 'react'

import { ExtendedVersion } from '@/bindings/ExtendedVersion'
import { extendedVersion } from '@/context/Functions'
import { flex, flex_col, items_center, min_h_0 } from '@/styles/Classes'
import Bowser from "bowser"

import { ScrollArea } from './ScrollArea'

const icon_style = css`
  width: 80px;
  height: 80px;
  border-radius: 16px;
`

const loading_wrap = css`
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 200px;
`

const space = css`
  .semi-descriptions-key {
    margin-right: 5px;
  }
`

interface AboutPanelProps {
  className?: string
}

export function AboutPanel({ className }: AboutPanelProps) {
  const [appVersion, setAppVersion] = useState('')
  const [tauriVersion, setTauriVersion] = useState('')
  const [extVersion, setExtVersion] = useState<ExtendedVersion | null>(null)

  const browser = useMemo(() => {
    return Bowser.getParser(navigator.userAgent)
  }, [navigator.userAgent])


  useEffect(() => {
    getVersion().then(setAppVersion)
    getTauriVersion().then(setTauriVersion)
    extendedVersion(true).then(setExtVersion)
  }, [])

  const libraryData: Data[] = extVersion
    ? extVersion.linkedLibraries.map((lib) => ({
        key: lib.name,
        value: lib.runtimeVersion
          ? `${lib.compiledVersion} (runtime: ${lib.runtimeVersion})`
          : lib.compiledVersion,
      }))
    : []

  const loadedLibraryData: Data[] = extVersion
    ? extVersion.loadedLibraries.map((lib) => ({
        key: lib.name,
        value: lib.version,
      }))
    : []

  if (!extVersion) {
    return (
      <div className={loading_wrap}>
        <Spin />
      </div>
    )
  }

  return (
    <div className={cx(flex, flex_col, min_h_0, space, className)}>
      <div className={cx(flex, flex_col, items_center)}>
        <img src="/svnexus-icon.svg" alt="SVNexus" className={icon_style} />
        <Typography.Title heading={4} style={{ marginTop: 12 }}>
          SVNexus
        </Typography.Title>
        <Typography.Text type="tertiary">Version {appVersion}</Typography.Text>
        <Typography.Text style={{ marginTop: 12 }}>
          A modern Subversion client built with Tauri
        </Typography.Text>
      </div>

      <Divider style={{ margin: '16px 0' }} />

      <ScrollArea className={cx(min_h_0)}>
        <div className={cx(flex, flex_col)}>
          <Descriptions
            data={[
              {
                key: 'Subversion',
                value: `${extVersion.version.major}.${extVersion.version.minor}.${extVersion.version.patch}${extVersion.version.tag}(${extVersion.buildDate} ${extVersion.buildTime})`,
              },
              { key: 'Tauri', value: tauriVersion },
              { key: 'WebView', value: `${browser.getEngine().name} ${browser.getEngine().version}` },
              { key: 'Host', value: extVersion.buildHost },
              { key: 'Runtime Host', value: extVersion.runtimeHost },
            ]}
          />
        </div>
        <div>
          {libraryData.length > 0 && (
            <>
              <Divider style={{ width: '100%', margin: '12px 0' }} />
              <Typography.Text type="tertiary" size="small">
                Linked Libraries
              </Typography.Text>
              <Descriptions data={libraryData} style={{ marginTop: 8 }} />
            </>
          )}

          {loadedLibraryData.length > 0 && (
            <>
              <Divider style={{ width: '100%', margin: '12px 0' }} />
              <Typography.Text type="tertiary" size="small">
                Loaded Libraries
              </Typography.Text>
              <Descriptions data={loadedLibraryData} style={{ marginTop: 8 }} />
            </>
          )}
        </div>
      </ScrollArea>
    </div>
  )
}
