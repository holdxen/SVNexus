import './styles/fonts.css'
// main.tsx
import ReactDOM from 'react-dom/client'

import App from './App'
import { useSettings } from './context/Settings'
import './Style'

import '@styles/hover.css'
import 'react-responsive-modal/styles.css'

async function bootstrap() {
  try {
    // 先读配置并应用主题、语言，再渲染，避免先闪一下默认外观
    await useSettings.getState().load()
  } catch (error) {
    // 读配置失败就按默认设置启动，不阻塞界面
    console.warn('Failed to load settings:', error)
  }

  ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(<App />)
}

bootstrap()
