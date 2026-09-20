import '@douyinfe/semi-ui/react19-adapter'
import { ConfigProvider, Divider, Dropdown } from '@douyinfe/semi-ui'
import MoreMenuIcon from '@icons/MoreMenu.svg?react'
import RecordsIcon from '@icons/Records.svg?react'
import { css, cx } from '@linaria/core'
// import {
//   writeText as tauriWriteText,
//   readText as tauriReadText,
// } from '@tauri-apps/plugin-clipboard-manager'
import { useMemoizedFn } from 'ahooks'
import { useEffect, useMemo, useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router'
import * as uuid from 'uuid'

import { AboutPanel } from './components/AboutPanel'
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
  const [activeIdentity, setActiveIdentity] = useState<string | number>('')
  const [tabs, setTabs] = useState<TabViewModel[]>([])
  const modal = useModal()

  const showAboutDialog = () => {
    modal.show(NiceAboutDialog, {})
  }

  const closeTab = useMemoizedFn((identity: Identity) => {
    const index = tabs.findIndex((i) => i.identity === identity)
    if (index < 0) {
      return
    }
    if (activeIdentity === identity) {
      if (tabs.length <= 1) {
        setActiveIdentity('')
      } else {
        if (index === 0) {
          setActiveIdentity(tabs[1].identity)
        } else {
          setActiveIdentity(tabs[index - 1].identity)
        }
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
      title: 'Welcome',
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
  const goTo = useMemoizedFn((identity: Identity) => {
    if (tabs.findIndex((i) => i.identity === identity) < 0) {
      Logger.warn('No such tab:', identity)
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
          title: 'Workspace',
          content,
        }
        setupModel(model)
        setTabs((items) => [...items, model])
        setActiveIdentity(identity)
      },
      closeOnly: (identity: Identity) => {
        setTimeout(() => {
          setTabs((items) => items.filter((e) => e.identity !== identity))
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
    <ConfigProvider>
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
                    <Dropdown.Item onClick={showAboutDialog}>About</Dropdown.Item>
                  </Dropdown.Menu>
                }
              >
                <IconButton>
                  <MoreMenuIcon></MoreMenuIcon>
                </IconButton>
              </Dropdown>
              <IconButton>
                <RecordsIcon></RecordsIcon>
              </IconButton>
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
