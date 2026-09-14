import { Divider, Toast } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import { useMemoizedFn } from 'ahooks'
import React, {
  createContext,
  useContext,
  useEffect,
  useImperativeHandle,
  useRef,
  useState,
} from 'react'
import * as uuid from 'uuid'

import { UpgradeOptions } from '@/bindings/UpgradeOptions'
import { WorkspaceItem } from '@/bindings/WorkspaceItem'
import { useDatabase } from '@/context/Database'
import { Subversion, useSubversion } from '@/context/Subversion'
import { Identity, useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { ModalProvider, useCurrentModal, useModal } from '@/lib/multi-modal'
import errorHumanString, { isSubversionError } from '@/utils/Error'
import { localPath } from '@/utils/Path'
import { workspaceItemGetWorkingCopy } from '@/utils/WorkspaceItem'

import Container from '../../components/Container'
import {
  grid,
  h_full,
  min_h_0,
  hidden,
  flex_1,
  grid_rows_auto_auto_1fr,
  min_w_0,
  break_all,
  break_word,
} from '../../styles/Classes'
import ConfirmDialog from '../dialogs/ConfirmDialog'
import { Dialog } from '../dialogs/Dialog'
import { SubversionProvider } from '../SubversionProvider'
import { NavigationBar } from './NavigationBar'
import { WorkingCopyView } from './WorkingCopyView'
import { useWorkingspaceViewStore } from './WorkspaceView.Store'
import Logger from '@/utils/Logger'

const GoToDialog = (props: { children?: React.ReactNode }) => {
  const modal = useCurrentModal()

  return (
    <Dialog
      visible={modal.visible}
      title="Question"
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      afterClose={() => modal.remove()}
    >
      {props.children}
    </Dialog>
  )
}

export interface WorkspaceContext {
  path: string
  // subversion: number
}

const WorkspaceViewContext = createContext<WorkspaceContext | null>(null)

export function useWorkspaceContext(): WorkspaceContext {
  const dialog = useContext(WorkspaceViewContext)

  if (dialog === null) {
    throw new Error('No workspace context provided')
  }

  return dialog
}

// export interface WorkingCopyViewProps {
//   path: string
// }

interface WorkspaceViewProps {
  from: Identity
  path: string
}

export function WorkspaceView(props: WorkspaceViewProps) {
  return (
    <SubversionProvider className={cx(h_full)} singleton={true}>
      <ModalProvider>
        <WorkspaceViewInner {...props}></WorkspaceViewInner>
      </ModalProvider>
    </SubversionProvider>
  )
}

export interface WorkspaceViewRef {
  active: () => void
  switch: (path: string) => void
  identity: () => Identity
}

function WorkspaceViewInner({ from, path }: WorkspaceViewProps) {
  const subversion = useSubversion()
  const [root, setRoot] = useState<string | null>(null)

  const [current, setCurrent] = useState<string | null>(null)

  const [views, setViews] = useState(new Set<string>())

  const self = useRef<WorkspaceViewRef>(null)
  const tabManager = useTabManager()
  const tabContent = useTabContent()
  const addWorkspaceItem = useDatabase((state) => state.addWorkspaceItem)
  const workspaceItems = useDatabase((state) => state.workspaceItems)
  const updateWorkspaceItem = useDatabase((state) => state.updateWorkspaceItem)

  const currentWorkspaceItem = useRef<WorkspaceItem>(null)

  const modal = useModal()
  const handleError = useMemoizedFn(async (error: any) => {
    Logger.warn('Error: ', error)
    if (isSubversionError(error)) {
      if (error.subversionError.source.code === 'wcNotWorkingCopyOrDirectory') {
        const target = root !== null ? root : path
        await modal
          .show(ConfirmDialog, {
            title: 'Error',
            children: (
              <div
                className={cx(break_all, break_word)}
              >{`${target} is not a working copy folder`}</div>
            ),
            icon: 'error',
          })
          .as<boolean>()
        tabManager.goTo(from)
        tabManager.closeOnly(tabContent.identity)
      } else if (error.subversionError.source.code === 'wcUpgradeRequired') {
        const target = root !== null ? root : path
        const result = await modal
          .show(ConfirmDialog, {
            title: 'Upgrade',
            children: (
              <div className={cx(break_all, break_word)}>{`${target} upgrade required`}</div>
            ),
            icon: 'error',
          })
          .as<boolean>()
        if (result) {
          const options: UpgradeOptions = {
            path: target,
          }
          await Subversion.callOnce({
            factory: subversion,
            async call(context) {
              await context.upgrade(options)
            },
          })
          tabManager.reload(tabContent.identity)
        } else {
          tabManager.goTo(from)
          tabManager.closeOnly(tabContent.identity)
        }
      } else {
        Toast.error({
          stack: true,
          content: errorHumanString(error)
        })
      }
    }
  })

  const workspaceViewStore = useWorkingspaceViewStore()
  const success = useRef(false)

  const cleanup = useMemoizedFn(() => {
    if (success.current) {
      workspaceViewStore.openWorkingCopyQueue.run(() => {
        if (root) {
          workspaceViewStore.removeOpenedWorkingCopy(root)
        }
      })
    }
  })

  useEffect(() => {
    Logger.info('workspace start', path)
    async function call() {
      await Subversion.callOnce({
        factory: subversion,
        onError: (error) => {
          handleError(error)
        },
        async call(context) {
          const root = await context.getWcRoot(path)

          const view = workspaceViewStore.openedWorkingCopy.get(root)
          if (view !== undefined && view.current?.identity() !== tabContent.identity) {
            const result = await modal
              .show(GoToDialog, {
                children: (
                  <div
                    className={cx(break_all, break_word)}
                  >{`${path} has already been opened\nWhether go to the view`}</div>
                ),
              })
              .as<boolean>()
            if (result) {
              if (view.current) {
                tabManager.goTo(view.current.identity())
                view.current.switch(path)
              }
            } else {
              tabManager.goTo(from)
            }
            tabManager.closeOnly(tabContent.identity)
            return
          }

          success.current = true

          setRoot(root)

          setViews((items) => new Set(items).add(path))
          setCurrent(path)
          workspaceViewStore.addOpendWorkingCopy(root, self)

          const workspaceItem = workspaceItems.find(
            (e) => 'workingCopy' in e && e.workingCopy.workingCopyRoot === root,
          )

          let item = undefined

          if (workspaceItem) {
            item = workspaceItemGetWorkingCopy(workspaceItem)
          }
          Logger.info('ready to update workspace item')

          const name = item?.name ?? localPath.getFileName(root) ?? 'Untitled'
          currentWorkspaceItem.current = await addWorkspaceItem(async (order) => {
            return {
              workingCopy: {
                workingCopyRoot: root,
                workingCopyPath: path,
                repositoryRootUrl: item?.repositoryRootUrl ?? null,
                lastUsedTime: new Date().getTime(),
                checkout: item?.checkout ?? null,
                order,
                star: item?.star ?? false,
                identity: item?.identity ?? uuid.v4(),
                remark: item?.remark ?? null,
                name,
              },
            }
          }, item !== undefined)

          tabManager.setTitle(tabContent.identity, name)
          tabManager.setTooltip(tabContent.identity, root)
        },
      })
    }

    workspaceViewStore.openWorkingCopyQueue.run(call)

    return cleanup
  }, [])

  const onSwitched = (selected: string) => {
    if (!views.has(selected)) {
      setViews((v) => new Set(v).add(selected))
    }
    setCurrent(selected)
  }

  useImperativeHandle(self, () => ({
    active() {
      tabManager.goTo(tabContent.identity)
    },
    switch(path) {
      onSwitched(path)
    },
    identity() {
      return tabContent.identity
    },
  }))

  useEffect(() => {
    if (current && currentWorkspaceItem.current) {
      if ('workingCopy' in currentWorkspaceItem.current) {
        const wc = currentWorkspaceItem.current.workingCopy
        wc.workingCopyPath = current
        updateWorkspaceItem(currentWorkspaceItem.current)
      }
    }
  }, [current])

  if (root === null) {
    return <></>
  }

  return (
    <WorkspaceViewContext.Provider value={{ path: root }}>
      <div className={cx(grid, flex_1, min_h_0, min_w_0, grid_rows_auto_auto_1fr)}>
        {current === null ? (
          <div></div>
        ) : (
          <NavigationBar onSelected={onSwitched} path={current} root={root}></NavigationBar>
        )}
        <Divider></Divider>
        <Container className={cx(min_h_0, min_w_0)}>
          {Array.from(views).map((v) => {
            return (
              <Container className={cx(min_h_0, min_w_0, current !== v && hidden)} key={v}>
                <WorkingCopyView path={v}></WorkingCopyView>
              </Container>
            )
          })}
        </Container>
      </div>
    </WorkspaceViewContext.Provider>
  )
}
