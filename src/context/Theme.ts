import { create } from 'zustand'

export type ThemeMode = 'light' | 'dark' | 'system'
export type ResolvedTheme = 'light' | 'dark'

const DARK_QUERY = '(prefers-color-scheme: dark)'

function systemTheme(): ResolvedTheme {
  return window.matchMedia(DARK_QUERY).matches ? 'dark' : 'light'
}

function applyTheme(theme: ThemeMode): ResolvedTheme {
  const resolved = theme === 'system' ? systemTheme() : theme
  // Semi Design 的明暗变量挂在 body[theme-mode] 上，改这个属性即可切换整套配色
  document.body.setAttribute('theme-mode', resolved)
  return resolved
}

export interface ThemeStore {
  /** 用户的选择，可能是「跟随系统」 */
  theme: ThemeMode
  /** 实际生效的明暗，跟随系统时由系统偏好决定 */
  resolved: ResolvedTheme
  setTheme: (theme: ThemeMode) => void
}

export const useTheme = create<ThemeStore>()((set) => ({
  theme: 'light',
  resolved: 'light',
  setTheme: (theme) => set({ theme, resolved: applyTheme(theme) }),
}))

// 选了「跟随系统」时，系统在明暗之间切换要即时反映
window.matchMedia(DARK_QUERY).addEventListener('change', () => {
  if (useTheme.getState().theme === 'system') {
    useTheme.setState({ resolved: applyTheme('system') })
  }
})
