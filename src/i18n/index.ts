import { create } from 'zustand'

import { advancedDialogs } from './messages/advancedDialogs'
import { app } from './messages/app'
import { changes } from './messages/changes'
import { common } from './messages/common'
import { components } from './messages/components'
import { conflict } from './messages/conflict'
import { dialogs } from './messages/dialogs'
import { feedback } from './messages/feedback'
import { merge } from './messages/merge'
import { settings } from './messages/settings'
import { shared } from './messages/shared'
import { svnDialogs } from './messages/svnDialogs'
import { welcome } from './messages/welcome'
import { welcomeUi } from './messages/welcomeUi'
import { workspace } from './messages/workspace'

export type Locale = 'zh-CN' | 'en-US'
export type LocalePreference = Locale | 'system'

/** 语言名称用它自己的语言书写，切换列表里不随当前语言变化 */
export const LOCALES: { value: Locale; label: string }[] = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en-US', label: 'English' },
]

function systemLocale(): Locale {
  return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

function resolveLocale(preference: LocalePreference): Locale {
  return preference === 'system' ? systemLocale() : preference
}

const zh = {
  ...shared.zh,
  ...common.zh,
  ...app.zh,
  ...settings.zh,
  ...dialogs.zh,
  ...welcome.zh,
  ...workspace.zh,
  ...conflict.zh,
  ...feedback.zh,
  ...merge.zh,
  ...changes.zh,
  ...svnDialogs.zh,
  ...advancedDialogs.zh,
  ...welcomeUi.zh,
  ...components.zh,
}

const en: Record<keyof typeof zh, string> = {
  ...shared.en,
  ...common.en,
  ...app.en,
  ...settings.en,
  ...dialogs.en,
  ...welcome.en,
  ...workspace.en,
  ...conflict.en,
  ...feedback.en,
  ...merge.en,
  ...changes.en,
  ...svnDialogs.en,
  ...advancedDialogs.en,
  ...welcomeUi.en,
  ...components.en,
}

export type MessageKey = keyof typeof zh

const dictionaries: Record<Locale, Record<MessageKey, string>> = {
  'zh-CN': zh,
  'en-US': en,
}

export type MessageParams = Record<string, string | number>

export function translate(locale: Locale, key: MessageKey, params?: MessageParams): string {
  let text = dictionaries[locale][key]
  if (params) {
    for (const name of Object.keys(params)) {
      text = text.split(`{${name}}`).join(String(params[name]))
    }
  }
  return text
}

export interface LocaleStore {
  /** 用户的选择，可能是「跟随系统」 */
  preference: LocalePreference
  /** 实际生效的语言 */
  locale: Locale
  setLocale: (preference: LocalePreference) => void
}

// 语言目前只在会话内生效，尚未持久化
export const useLocale = create<LocaleStore>()((set) => ({
  preference: 'zh-CN',
  locale: 'zh-CN',
  setLocale: (preference) => set({ preference, locale: resolveLocale(preference) }),
}))

/** 组件里用：语言切换会触发重渲染 */
export function useT() {
  const locale = useLocale((state) => state.locale)
  return (key: MessageKey, params?: MessageParams) => translate(locale, key, params)
}

/** 事件回调、工具模块等非渲染场景用：读取当前语言的即时快照 */
export function t(key: MessageKey, params?: MessageParams): string {
  return translate(useLocale.getState().locale, key, params)
}
