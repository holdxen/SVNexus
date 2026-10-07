import '@douyinfe/semi-ui/react19-adapter'
import { ConfigProvider, Divider, Dropdown, Popover, Toast } from '@douyinfe/semi-ui'
import semiEnUS from '@douyinfe/semi-ui/lib/es/locale/source/en_US'
import semiZhCN from '@douyinfe/semi-ui/lib/es/locale/source/zh_CN'
import MoreMenuIcon from '@icons/MoreMenu.svg?react'
import RecordsIcon from '@icons/Records.svg?react'
import UpgradeIcon from '@icons/Upgrade.svg?react'
import { css, cx } from '@linaria/core'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'
// import {
//   writeText as tauriWriteText,
//   readText as tauriReadText,
// } from '@tauri-apps/plugin-clipboard-manager'
import { save } from '@tauri-apps/plugin-dialog'
import { check, Update } from '@tauri-apps/plugin-updater'
import { useMemoizedFn } from 'ahooks'
import { useEffect, useMemo, useRef, useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router'
import * as uuid from 'uuid'

import { useLocale, useT, type MessageKey } from '@/i18n'

import { AboutPanel } from './components/AboutPanel'
import WorkingCopyNotifyPanel from './components/subversion/WorkingCopyNotifyPanel'
import { logsExport } from './context/Functions'
import { useNotifyLogUnread } from './context/NotifyLog'
import { Identity, TabContentContextProvider } from './context/TabContent'
import { TabManager, TabManagerContext, TabViewModel } from './context/TabManager'
import { IconButton } from './icons/IconButton'
import { ModalProvider, useModal } from './lib/multi-modal'
import {
  flex,
  flex_col,
  h_full,
  min_h_0,
  flex_1,
  border_box,
  p_2,
  relative,
  items_center,
  flex_row_reverse,
  px_1,
  w_full,
  p_4,
} from './styles/Classes'
import { TabContent, Tab } from './tab/Tab'
import Logger from './utils/Logger'
import { NiceAboutDialog } from './views/dialogs/AboutDialog'
import { NiceAppUpdateDialog } from './views/dialogs/AppUpdateDialog'
import { NiceFeedbackDialog } from './views/dialogs/FeedbackDialog'
import { NiceSettingsDialog } from './views/dialogs/SettingsDialog'
import { WelcomeView } from './views/WelcomeView/WelcomeView'
import { RouteFileHistoryView } from './views/WorkspaceView/FileHitoryView'
import { WorkspaceView } from './views/WorkspaceView/WorkspaceView'

// // 将 navigator.clipboard 代理到 Tauri 后端，绕过 WKWebView 的剪贴板权限限制
// // 解决 Monaco Editor 在 WKWebView 中的 NotAllowedError 和 Canceled 错误
// if (navigator.clipboard) {
//   const original = navigator.clipboard
//   const proxy: Clipboard = {
//     writeText: (text: string) => tauriWriteText(text),
//     readText: () => tauriReadText(),
//     write: async (items: ClipboardItems) => {
//       for (const item of items) {
//         if (item.types.includes('text/plain')) {
//           const blob = await item.getType('text/plain')
//           const text = await blob.text()
//           await tauriWriteText(text)
//           return
//         }
//       }
//     },
//     read: original.read?.bind(original) as Clipboard['read'],
//     addEventListener: original.addEventListener.bind(original),
//     removeEventListener: original.removeEventListener.bind(original),
//     dispatchEvent: original.dispatchEvent.bind(original),
//   }
//   Object.defineProperty(navigator, 'clipboard', { value: proxy, configurable: true })
// }

export type TabContentModel =
  | {
      welcomeView: {}
    }
  | {
      workspaceView: {
        from: Identity
        path: string
      }
    }

/** 占位标题存成元素而不是字符串：语言切换时才会按当前语言重新渲染 */
function TabTitle({ messageKey }: { messageKey: MessageKey }) {
  const t = useT()
  return <>{t(messageKey)}</>
}

const unreadDot = css`
  position: absolute;
  top: 1px;
  right: 1px;
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: var(--semi-color-danger);
`

function TabContentView({ content }: { content: TabContentModel }) {
  if ('welcomeView' in content) {
    return <WelcomeView></WelcomeView>
  } else if ('workspaceView' in content) {
    return (
      <WorkspaceView
        from={content.workspaceView.from}
        path={content.workspaceView.path}
      ></WorkspaceView>
    )
  }
  return <></>
}

function App() {
  return (
    <ModalProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/FileHistoryView/:url/:revision" element={<RouteFileHistoryView />}></Route>
          <Route
            path="/About"
            element={<AboutPanel className={cx(w_full, h_full, border_box, p_4)}></AboutPanel>}
          ></Route>
        </Routes>
      </BrowserRouter>
    </ModalProvider>
  )
}

function Home() {
  const t = useT()
  const locale = useLocale((state) => state.locale)
  const [activeIdentity, setActiveIdentity] = useState<string | number>('')
  const [tabs, setTabs] = useState<TabViewModel[]>([])
  const [update, setUpdate] = useState<Update | null>(null)
  const [notifyLogVisible, setNotifyLogVisible] = useState(false)
  const modal = useModal()

  // 通知日志跟着 tab 走。欢迎页和工作副本页都挂了 SubversionProvider，
  // 所以两种 tab 都会有记录（包括欢迎页那些后台状态轮询产生的）
  const activeTab = tabs.find((tab) => tab.identity === activeIdentity)
  const notifyLogIdentity =
    activeTab !== undefined &&
    ('workspaceView' in activeTab.content || 'welcomeView' in activeTab.content)
      ? activeTab.identity
      : null
  const hasUnreadNotify = useNotifyLogUnread(notifyLogIdentity)

  // 面板内容跟着激活的 tab 变，切换 tab 时若继续开着，用户会看到日志被无声换掉
  useEffect(() => {
    setNotifyLogVisible(false)
  }, [activeIdentity])

  const showAboutDialog = () => {
    modal.show(NiceAboutDialog, {})
  }

  const showSettingsDialog = () => {
    modal.show(NiceSettingsDialog, {})
  }

  const showUpdateDialog = () => {
    if (!update) {
      return
    }
    modal.show(NiceAppUpdateDialog, { update })
  }

  const showFeedbackDialog = () => {
    // 当前激活的工作副本排在最前，便于对方定位问题发生在哪个工作副本
    const active = tabs.find((tab) => tab.identity === activeIdentity)
    const activePath =
      active && 'workspaceView' in active.content ? active.content.workspaceView.path : undefined
    const paths = tabs.flatMap((tab) =>
      'workspaceView' in tab.content ? [tab.content.workspaceView.path] : [],
    )
    modal.show(NiceFeedbackDialog, {
      workingCopies: activePath
        ? [activePath, ...paths.filter((path) => path !== activePath)]
        : paths,
    })
  }

  // 弹系统保存对话框选导出路径，后端把日志目录整体压成 zip 写过去
  const exportLogs = async () => {
    try {
      const path = await save({
        title: t('dialogs.saveFile'),
        defaultPath: 'svnexus-logs.zip',
        filters: [{ name: 'Zip', extensions: ['zip'] }],
      })
      if (!path) {
        return
      }
      await logsExport(path)
      Toast.success({ content: t('app.exportLogs.success'), stack: true })
    } catch (e) {
      Logger.warn('Export logs failed: ', e)
      Toast.error({ content: t('app.exportLogs.failed'), stack: true })
    }
  }

  // 启动时检查更新；有新版本才显示升级图标
  useEffect(() => {
    if (import.meta.env.DEV) {
      return
    }
    check()
      .then((available) => {
        if (available) {
          setUpdate(available)
        }
      })
      .catch((e) => {
        Logger.info('Update check failed: ', e)
      })
  }, [])

  // 手动检查更新：有更新直接弹窗并显示升级图标，无更新提示已最新
  const checkingRef = useRef(false)
  const checkUpdateManually = async () => {
    if (update) {
      modal.show(NiceAppUpdateDialog, { update })
      return
    }
    if (checkingRef.current) {
      return
    }
    checkingRef.current = true
    try {
      const available = await check()
      if (available) {
        setUpdate(available)
        modal.show(NiceAppUpdateDialog, { update: available })
      } else {
        Toast.success({ content: t('app.update.upToDate'), stack: true })
      }
    } catch (e) {
      Logger.warn('Manual update check failed: ', e)
      Toast.error({ content: t('app.update.checkFailed'), stack: true })
    } finally {
      checkingRef.current = false
    }
  }

  const closeTab = useMemoizedFn((identity: Identity) => {
    const index = tabs.findIndex((i) => i.identity === identity)
    if (index < 0) {
      return
    }

    // 如果关闭的是最后一个 tab，直接替换成新的 welcome tab
    if (tabs.length === 1) {
      const newIdentity = uuid.v4()
      const content: TabContentModel = {
        welcomeView: {},
      }
      const model: TabViewModel = {
        identity: newIdentity,
        title: <TabTitle messageKey="app.tab.welcome" />,
        onClose: () => {
          closeTab(newIdentity)
        },
        onClick: () => {
          setActiveIdentity(newIdentity)
        },
        content,
      }
      setTabs([model])
      setActiveIdentity(newIdentity)
      return
    }

    // 关闭的不是最后一个 tab，正常处理
    if (activeIdentity === identity) {
      if (index === 0) {
        setActiveIdentity(tabs[1].identity)
      } else {
        setActiveIdentity(tabs[index - 1].identity)
      }
    }
    setTimeout(() => {
      setTabs((items) => items.filter((_, i) => i !== index))
    }, 0)
  })

  // 添加新 tab
  const addTab = () => {
    const identity = uuid.v4()
    const content: TabContentModel = {
      welcomeView: {},
    }
    const model: TabViewModel = {
      identity,
      title: <TabTitle messageKey="app.tab.welcome" />,
      onClose: () => {
        closeTab(identity)
      },
      onClick: () => {
        setActiveIdentity(identity)
      },
      content,
    }
    setTabs((items) => [...items, model])
    setActiveIdentity(identity)
  }

  useEffect(() => {
    addTab()
  }, [])

  // 处理 CLI 传入的路径（类似 VSCode 的 `code /path`）
  const openPathFromCli = useMemoizedFn((path: string) => {
    // 使用一个虚拟的 identity 作为 "from"，因为 CLI 没有来源 tab
    const fromIdentity = 'cli'
    tabManager.openWorkingCopy(fromIdentity, path)
  })

  useEffect(() => {
    // 获取首次启动时传入的路径
    invoke<string | null>('cli_initial_path')
      .then((path) => {
        if (path) {
          Logger.info('CLI initial path:', path)
          openPathFromCli(path)
        }
      })
      .catch((e) => {
        Logger.warn('Failed to get CLI initial path:', e)
      })

    // 监听后续的单实例转发事件
    const unlisten = listen<string>('cli-open-path', (event) => {
      Logger.info('CLI open-path event:', event.payload)
      openPathFromCli(event.payload)
    })

    return () => {
      unlisten.then((fn) => fn())
    }
  }, [openPathFromCli])

  const goToLast = useMemoizedFn(() => {
    if (tabs.length === 0) {
      return
    }
    setActiveIdentity(tabs[tabs.length - 1].identity)
  })

  const setupModel = (model: TabViewModel) => {
    model.onClick = () => {
      setActiveIdentity(model.identity)
    }
    model.onClose = () => {
      closeTab(model.identity)
      // setTabs((items) => items.filter((i) => i.identity !== model.identity))
    }
  }
  const goTo = useMemoizedFn((identity: Identity, exclude?: Identity) => {
    if (tabs.findIndex((i) => i.identity === identity) < 0) {
      Logger.warn('No such tab:', identity)
      for (let tab of tabs.reverse()) {
        if (tab.identity === exclude) {
          continue
        }
        setActiveIdentity(tab.identity)
        break
      }
      return
    }
    setActiveIdentity(identity)
  })

  const tabManager: TabManagerContext = useMemo(
    () => ({
      add: (model, jump) => {
        setupModel(model)
        setTabs((items) => [...items, model])
        if (jump) {
          setActiveIdentity(model.identity)
        }
      },
      goTo,
      goToLast,
      setTitle: (identity, title) => {
        setTabs((items) => {
          for (let i of items) {
            if (i.identity === identity) {
              i.title = title
            }
          }
          return [...items]
        })
      },
      setTooltip: (identity, tooltip) => {
        setTabs((items) => {
          for (let i of items) {
            if (i.identity === identity) {
              i.tooltip = tooltip
            }
          }
          return [...items]
        })
      },
      close: (identity) => {
        closeTab(identity)
      },
      openWorkingCopy(from, path) {
        path = path.replace(/\\/g, '/')
        const identity = uuid.v4()
        const content: TabContentModel = {
          workspaceView: {
            path,
            from,
          },
        }
        const model: TabViewModel = {
          identity,
          title: <TabTitle messageKey="app.tab.workspace" />,
          content,
        }
        setupModel(model)
        setTabs((items) => [...items, model])
        setActiveIdentity(identity)
      },
      closeOnly: (identity: Identity) => {
        setTimeout(() => {
          setTabs((items) => {
            const remaining = items.filter((e) => e.identity !== identity)
            if (remaining.length === 0) {
              const newIdentity = uuid.v4()
              const model: TabViewModel = {
                identity: newIdentity,
                title: <TabTitle messageKey="app.tab.welcome" />,
                onClose: () => {
                  closeTab(newIdentity)
                },
                onClick: () => {
                  setActiveIdentity(newIdentity)
                },
                content: { welcomeView: {} },
              }
              setActiveIdentity(newIdentity)
              return [model]
            }
            return remaining
          })
        }, 0)
      },
      reload: (identity: Identity) => {
        setTimeout(() => {
          setTabs((items) => {
            for (let i of items) {
              if (i.identity === identity) {
                i.identity = uuid.v4()
              }
            }
            return [...items]
          })
        }, 0)
      },
    }),
    [],
  )

  return (
    <ConfigProvider locale={locale === 'en-US' ? semiEnUS : semiZhCN}>
      <TabManager.Provider value={tabManager}>
        <main
          style={{ backgroundColor: 'var(--svnexus-base-color)' }}
          className={cx(flex, flex_col, h_full, min_h_0)}
        >
          <div className={cx(flex)}>
            <Tab
              onReordered={(models) => setTabs(models as TabViewModel[])}
              className={cx(
                border_box,
                p_2,
                css`
                  flex: 1 1 auto;
                `,
              )}
              models={tabs}
              activeIdentity={activeIdentity}
              onAdd={addTab}
            />
            <div
              className={cx(
                border_box,
                px_1,
                flex,
                items_center,
                flex_row_reverse,
                css`
                  flex: 0 0 85px;
                `,
              )}
            >
              <Dropdown
                clickToHide
                trigger="click"
                render={
                  <Dropdown.Menu>
                    <Dropdown.Item onClick={showSettingsDialog}>
                      {t('app.menu.settings')}
                    </Dropdown.Item>
                    <Dropdown.Item onClick={checkUpdateManually}>
                      {t('app.menu.checkUpdate')}
                    </Dropdown.Item>
                    <Dropdown.Item onClick={exportLogs}>{t('app.menu.exportLogs')}</Dropdown.Item>
                    <Dropdown.Item onClick={showFeedbackDialog}>
                      {t('app.menu.feedback')}
                    </Dropdown.Item>
                    <Dropdown.Item onClick={showAboutDialog}>{t('app.menu.about')}</Dropdown.Item>
                  </Dropdown.Menu>
                }
              >
                <IconButton>
                  <MoreMenuIcon></MoreMenuIcon>
                </IconButton>
              </Dropdown>
              {notifyLogIdentity === null ? (
                <IconButton className="inactive">
                  <RecordsIcon></RecordsIcon>
                </IconButton>
              ) : (
                // 自己管开关，不用 Semi 的 clickToHide / clickTriggerToHide：
                // clickToHide 的语义是"点弹层内部任何地方都收起"，会把点行、点展开、点清除都算进去；
                // clickTriggerToHide 只在已打开时能收起，关掉之后再点触发器无法重新打开。
                // trigger="custom" 配合下面两个回调，四种行为才都对。
                <Popover
                  trigger="custom"
                  position="bottomRight"
                  showArrow={false}
                  visible={notifyLogVisible}
                  onClickOutSide={() => setNotifyLogVisible(false)}
                  content={
                    <WorkingCopyNotifyPanel
                      identity={notifyLogIdentity}
                      visible={notifyLogVisible}
                    />
                  }
                >
                  <IconButton
                    style={{ position: 'relative' }}
                    onClick={() => setNotifyLogVisible((visible) => !visible)}
                  >
                    <RecordsIcon></RecordsIcon>
                    {hasUnreadNotify ? <span className={unreadDot}></span> : null}
                  </IconButton>
                </Popover>
              )}
              {update ? (
                <IconButton onClick={showUpdateDialog}>
                  <UpgradeIcon></UpgradeIcon>
                </IconButton>
              ) : null}
            </div>
          </div>
          <Divider></Divider>
          <div className={cx(flex_1, min_h_0, relative)}>
            {tabs.map((tab) => (
              <TabContentContextProvider.Provider
                key={tab.identity}
                value={{ identity: tab.identity }}
              >
                <TabContent
                  visible={tab.identity === activeIdentity}
                  // style={{ display: tab.key === activeKey ? 'grid' : 'none', height: '100%' }}
                >
                  <ModalProvider>
                    <TabContentView content={tab.content as TabContentModel}></TabContentView>
                  </ModalProvider>
                </TabContent>
              </TabContentContextProvider.Provider>
            ))}
          </div>
        </main>
      </TabManager.Provider>
    </ConfigProvider>
  )
}

export default App
