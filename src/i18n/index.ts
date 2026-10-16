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

export type Locale = 'zh-CN' | 'en-US' | 'es-419' | 'pt-BR' | 'ja-JP' | 'de-DE'
export type LocalePreference = Locale | 'system'

/** 语言名称用它自己的语言书写，切换列表里不随当前语言变化 */
export const LOCALES: { value: Locale; label: string }[] = [
  { value: 'zh-CN', label: '简体中文' },
  { value: 'en-US', label: 'English' },
  { value: 'es-419', label: 'Español' },
  { value: 'pt-BR', label: 'Português' },
  { value: 'ja-JP', label: '日本語' },
  { value: 'de-DE', label: 'Deutsch' },
]

function systemLocale(): Locale {
  const language = navigator.language.toLowerCase()
  if (language.startsWith('zh')) return 'zh-CN'
  if (language.startsWith('es')) return 'es-419'
  if (language.startsWith('pt')) return 'pt-BR'
  if (language.startsWith('ja')) return 'ja-JP'
  if (language.startsWith('de')) return 'de-DE'
  return 'en-US'
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

const es: Record<keyof typeof zh, string> = {
  ...shared.es,
  ...common.es,
  ...app.es,
  ...settings.es,
  ...dialogs.es,
  ...welcome.es,
  ...workspace.es,
  ...conflict.es,
  ...feedback.es,
  ...merge.es,
  ...changes.es,
  ...svnDialogs.es,
  ...advancedDialogs.es,
  ...welcomeUi.es,
  ...components.es,
}

const pt: Record<keyof typeof zh, string> = {
  ...shared.pt,
  ...common.pt,
  ...app.pt,
  ...settings.pt,
  ...dialogs.pt,
  ...welcome.pt,
  ...workspace.pt,
  ...conflict.pt,
  ...feedback.pt,
  ...merge.pt,
  ...changes.pt,
  ...svnDialogs.pt,
  ...advancedDialogs.pt,
  ...welcomeUi.pt,
  ...components.pt,
}

const ja: Record<keyof typeof zh, string> = {
  ...shared.ja,
  ...common.ja,
  ...app.ja,
  ...settings.ja,
  ...dialogs.ja,
  ...welcome.ja,
  ...workspace.ja,
  ...conflict.ja,
  ...feedback.ja,
  ...merge.ja,
  ...changes.ja,
  ...svnDialogs.ja,
  ...advancedDialogs.ja,
  ...welcomeUi.ja,
  ...components.ja,
}

const de: Record<keyof typeof zh, string> = {
  ...shared.de,
  ...common.de,
  ...app.de,
  ...settings.de,
  ...dialogs.de,
  ...welcome.de,
  ...workspace.de,
  ...conflict.de,
  ...feedback.de,
  ...merge.de,
  ...changes.de,
  ...svnDialogs.de,
  ...advancedDialogs.de,
  ...welcomeUi.de,
  ...components.de,
}

export type MessageKey = keyof typeof zh

const dictionaries: Record<Locale, Record<MessageKey, string>> = {
  'zh-CN': zh,
  'en-US': en,
  'es-419': es,
  'pt-BR': pt,
  'ja-JP': ja,
  'de-DE': de,
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
