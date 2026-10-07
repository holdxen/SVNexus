import { Button, Divider, FloatButton, Popover, TabPane, Tabs, Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { createContext, useContext, useState } from 'react'

import { NodeKind } from '@/bindings/NodeKind'
import LazyComponent from '@/components/LazyComponent'
import { ScrollArea } from '@/components/ScrollArea'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { useT } from '@/i18n'
import { repoPath } from '@/utils/Path'

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
  items_center,
  gap_x_1,
  gap_y_1,
  break_all,
  break_word,
  flex_shrink_0,
  text_12px,
  px_1,
} from '../../styles/Classes'
import { ChangesView } from './ChangesView/ChangesView'
import { CopyMoveItemsContext, CopyMoveSource, useCopyMoveItems } from './CopyMoveContext'
import { HistoryView } from './HistoryView/HistoryView'
import { RemoteView } from './RemoteView/RemoteView'

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

// 待复制/移动按钮固定在外层，作为 Popover 的锚点；按钮自身改为随锚点布局，
// 这样 Popover 对齐的是按钮本体，而不是 0 尺寸的包装元素
const anchor = css`
  position: fixed;
  right: 48px;
  bottom: 48px;
  z-index: 1000;
`

const popoverContent = css`
  width: 320px;
`

const itemList = css`
  max-height: 220px;
`

function CopyMoveSourceButton(props: {
  basePath: string
  source?: CopyMoveSource
  isChangedView: boolean
  isHistoryView: boolean
  isRemoteView: boolean
}) {
  const { setSource } = useCopyMoveItems()
  const t = useT()

  const source = props.source
  if (typeof source !== 'object') {
    return <></>
  }

  let action: string
  let items: { path: string; nodeKind?: NodeKind }[]
  if ('copy' in source && props.isHistoryView === false) {
    action = t('workspace.copyMove.copy')
    items = source.copy
  } else if (
    'move' in source &&
    ((source.isLocal && props.isChangedView) || (!source.isLocal && props.isRemoteView))
  ) {
    action = t('workspace.copyMove.move')
    items = source.move
  } else {
    return <></>
  }
  if (items.length === 0) {
    return <></>
  }

  const cancel = () => {
    setSource(undefined)
  }

  const preview = (
    <div className={cx(flex, flex_col, popoverContent)}>
      <Typography.Text strong className={cx(text_12px)}>
        {t('workspace.copyMove.pendingItems', { action, count: items.length })}
      </Typography.Text>
      <Divider margin="8px 0"></Divider>
      <ScrollArea className={cx(itemList)}>
        <div className={cx(flex, flex_col, gap_y_1)}>
          {items.map((item) => (
            <div
              key={item.path}
              className={cx(
                border_box,
                px_1,
                flex,
                items_center,
                gap_x_1,
                css`
                  svg {
                    width: 20px;
                    height: 20px;
                  }
                `,
              )}
            >
              <FileKindIcon
                kind={item.nodeKind ?? 'unknown'}
                className={cx(flex_shrink_0)}
              ></FileKindIcon>
              <span className={cx(flex_1, break_all, break_word, text_12px)}>
                {repoPath.stripPrefix(item.path, props.basePath) || item.path}
              </span>
            </div>
          ))}
        </div>
      </ScrollArea>
      <Divider margin="8px 0"></Divider>
      <Button size="small" type="tertiary" block onClick={cancel}>
        {t('workspace.copyMove.cancel')}
      </Button>
    </div>
  )

  return (
    <Popover
      trigger="hover"
      position="leftBottom"
      mouseEnterDelay={100}
      mouseLeaveDelay={300}
      content={preview}
    >
      <div className={cx(anchor)}>
        <FloatButton
          badge={{ count: items.length, overflowCount: 999 }}
          style={{
            position: 'relative',
            right: 'auto',
            bottom: 'auto',
            backgroundColor: 'rgba(var(--semi-grey-0), 1)',
          }}
          icon={
            <span style={{ fontSize: 10 }}>
              {action === t('workspace.copyMove.copy') ? 'Copy' : 'Move'}
            </span>
          }
        ></FloatButton>
      </div>
    </Popover>
  )
}

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

  const [copyMoveSource, setCopyMoveSource] = useState<CopyMoveSource>()

  return (
    <WorkingCopyViewContext.Provider
      value={{
        path,
        changesViewOperationContainer,
        historyViewOperationContainer,
        remoteViewOperationContainer,
      }}
    >
      <CopyMoveItemsContext.Provider
        value={{
          source: copyMoveSource,
          setSource: setCopyMoveSource,
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
            <LazyComponent visible={activeKey === remoteViewKey}>
              <RemoteView
                className={cx(flex_1, min_w_0, activeKey !== remoteViewKey && hidden)}
              ></RemoteView>
            </LazyComponent>
          </div>
          <CopyMoveSourceButton
            basePath={path}
            isChangedView={activeKey === changesViewKey}
            isHistoryView={activeKey === historyViewKey}
            isRemoteView={activeKey === remoteViewKey}
            source={copyMoveSource}
          ></CopyMoveSourceButton>
        </div>
      </CopyMoveItemsContext.Provider>
    </WorkingCopyViewContext.Provider>
  )
}
