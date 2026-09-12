import { Divider, TabPane, Tabs } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { createContext, useContext, useState } from 'react'

import LazyComponent from '@/components/LazyComponent'

import {
  min_h_0,
  flex,
  flex_1,
  border_box,
  m_1,
  h_auto,
  flex_col,
  hidden,
  min_w_0,
} from '../../styles/Classes'
import { ChangesView } from './ChangesView/ChangesView'
import { HistoryView } from './HistoryView/HistoryView'

export interface WorkingCopyContext {
  path: string
  changesViewOperationContainer: HTMLDivElement | null
  historyViewOperationContainer: HTMLDivElement | null
  remoteViewOperationContainer: HTMLDivElement | null
}

const WorkingCopyViewContext = createContext<WorkingCopyContext | null>(null)

export function useWorkingCopyContext(): WorkingCopyContext {
  const dialog = useContext(WorkingCopyViewContext)

  if (dialog === null) {
    throw new Error('No working copy context provided')
  }

  return dialog
}

export interface WorkingCopyViewProps {
  path: string
}

const bottom = css`
  box-shadow: inset 0 -1px 0 var(--semi-color-border);
`

const transparentBottom = css`
  --semi-color-border: transparent;
`

const contentPadding = css`
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  padding-top: 0px;
  padding-bottom: 0.5rem;
`

export function WorkingCopyView({ path }: WorkingCopyViewProps) {
  const changesViewKey = 'changes'
  const historyViewKey = 'history'
  const remoteViewKey = 'remote'
  const [activeKey, setActiveKey] = useState(changesViewKey)

  const [changesViewOperationContainer, setChangesViewOperationContainer] =
    useState<HTMLDivElement | null>(null)
  const [historyViewOperationContainer, setHistoryViewOperationContainer] =
    useState<HTMLDivElement | null>(null)
  const [remoteViewOperationContainer, setRemoteViewOperationContainer] =
    useState<HTMLDivElement | null>(null)

  return (
    <WorkingCopyViewContext.Provider
      value={{
        path,
        changesViewOperationContainer,
        historyViewOperationContainer,
        remoteViewOperationContainer,
      }}
    >
      <div className={cx(flex, contentPadding, min_h_0, min_w_0, flex_col, border_box)}>
        <Tabs
          tabPaneMotion={false}
          renderTabBar={(props, DefaultTabBar) => {
            const { tabBarExtraContent, ...restProps } = props
            return (
              <div className={cx(flex, bottom, border_box)}>
                <div className={cx(transparentBottom)}>
                  <DefaultTabBar {...restProps} />
                </div>
                <Divider layout="vertical" className={cx(m_1, h_auto)}></Divider>
                <div
                  ref={setChangesViewOperationContainer}
                  className={cx(flex_1, flex, activeKey !== changesViewKey && hidden)}
                >
                  {tabBarExtraContent}
                </div>
                <div
                  ref={setHistoryViewOperationContainer}
                  className={cx(flex_1, flex, activeKey !== historyViewKey && hidden)}
                >
                  {tabBarExtraContent}
                </div>
                <div
                  ref={setRemoteViewOperationContainer}
                  className={cx(flex_1, flex, activeKey !== remoteViewKey && hidden)}
                >
                  {tabBarExtraContent}
                </div>
              </div>
            )
          }}
          onChange={(key) => setActiveKey(key)}
          size="small"
          contentStyle={{ display: 'none' }}
          defaultActiveKey={changesViewKey}
        >
          <TabPane tab="Changes" itemKey={changesViewKey}></TabPane>
          <TabPane tab="History" itemKey={historyViewKey}></TabPane>
          <TabPane tab="Remote" itemKey={remoteViewKey}></TabPane>
        </Tabs>
        <div className={cx(flex_1, flex, min_h_0)}>
          <LazyComponent visible={activeKey === changesViewKey}>
            <ChangesView
              className={cx(min_h_0, flex_1, border_box, activeKey !== changesViewKey && hidden)}
            ></ChangesView>
          </LazyComponent>
          <LazyComponent visible={activeKey === historyViewKey}>
            <HistoryView
              className={cx(flex_1, min_w_0, activeKey !== historyViewKey && hidden)}
            ></HistoryView>
          </LazyComponent>
        </div>
      </div>
    </WorkingCopyViewContext.Provider>
  )
}
