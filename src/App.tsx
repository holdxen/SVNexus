import '@douyinfe/semi-ui/react19-adapter'
import { ConfigProvider, Divider, Dropdown } from '@douyinfe/semi-ui'
import MoreMenuIcon from '@icons/MoreMenu.svg?react'
import RecordsIcon from '@icons/Records.svg?react'
import { css, cx } from '@linaria/core'
import loader from '@monaco-editor/loader'
// import {
//   writeText as tauriWriteText,
//   readText as tauriReadText,
// } from '@tauri-apps/plugin-clipboard-manager'
import { useMemoizedFn } from 'ahooks'
import * as monaco from 'monaco-editor'
import editorWorker from 'monaco-editor/editor/editor.worker?worker'
import cssWorker from 'monaco-editor/language/css/css.worker?worker'
import htmlWorker from 'monaco-editor/language/html/html.worker?worker'
import jsonWorker from 'monaco-editor/language/json/json.worker?worker'
import tsWorker from 'monaco-editor/language/typescript/ts.worker?worker'
import { useEffect, useState } from 'react'
import { BrowserRouter, Routes, Route } from 'react-router'
import * as uuid from 'uuid'

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
} from './styles/Classes'
import { TabContent, Tab } from './tab/Tab'
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

self.MonacoEnvironment = {
  getWorker(_, label) {
    if (label === 'json') {
      return new jsonWorker()
    }
    if (label === 'css' || label === 'scss' || label === 'less') {
      return new cssWorker()
    }
    if (label === 'html' || label === 'handlebars' || label === 'razor') {
      return new htmlWorker()
    }
    if (label === 'typescript' || label === 'javascript') {
      return new tsWorker()
    }
    return new editorWorker()
  },
}

loader.config({ monaco })

function App() {
  return (
    <ModalProvider>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Home />} />
          <Route path="/FileHistoryView/:url/:revision" element={<RouteFileHistoryView />}></Route>
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
    const model: TabViewModel = {
      identity,
      title: 'Welcome',
      onClose: () => {
        console.log('close uuid', identity, tabs)
        closeTab(identity)
      },
      onClick: () => {
        console.log('set active: ', identity)
        setActiveIdentity(identity)
      },
      content: <WelcomeView></WelcomeView>,
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

  const tabManager: TabManagerContext = {
    add: (model, jump) => {
      setupModel(model)
      setTabs((items) => [...items, model])
      if (jump) {
        setActiveIdentity(model.identity)
      }
    },
    goTo: (identity) => {
      setTimeout(() => {
        if (tabs.findIndex((i) => i.identity === identity) < 0) {
          return
        }
        setActiveIdentity(identity)
      }, 0)
    },
    goToLast: () => {
      goToLast()
    },
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
            console.log('set tooltip: ', identity, tooltip)
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
      const identity = uuid.v4()
      const model: TabViewModel = {
        identity,
        title: 'Workspace',
        content: <WorkspaceView from={from} path={path}></WorkspaceView>,
      }
      setupModel(model)
      setTabs((items) => [...items, model])
      setActiveIdentity(identity)
      console.log('open workspace: ', from, path, identity)
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
  }

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
                  <ModalProvider>{tab.content}</ModalProvider>
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
