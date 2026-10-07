import { invoke } from '@tauri-apps/api/core'
import { create } from 'zustand'

import type { Settings } from '@/bindings/Settings'
import { useLocale } from '@/i18n'

import { useTheme } from './Theme'

/** 与 Rust 侧 `Settings::default()` 保持一致 */
export const defaultSettings: Settings = {
  theme: 'light',
  locale: 'zh-CN',
  defaultUsername: '',
  defaultPassword: '',
  proxyEnabled: false,
  proxyType: 'http',
  proxyHost: '',
  proxyPort: null,
  proxyUsername: '',
  proxyPassword: '',
}

export interface SettingsStore {
  settings: Settings
  /** 启动时读取配置文件并应用主题、语言 */
  load: () => Promise<void>
  /** 用户点「确定」时：写回配置文件，同时应用主题、语言 */
  save: (settings: Settings) => Promise<void>
}

export const useSettings = create<SettingsStore>()((set) => ({
  settings: defaultSettings,
  load: async () => {
    const settings = await invoke<Settings>('load_settings')
    useTheme.getState().setTheme(settings.theme)
    useLocale.getState().setLocale(settings.locale)
    set({ settings })
  },
  save: async (settings) => {
    await invoke('save_settings', { settings })
    useTheme.getState().setTheme(settings.theme)
    useLocale.getState().setLocale(settings.locale)
    set({ settings })
  },
}))
