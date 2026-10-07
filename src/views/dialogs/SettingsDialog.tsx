import { Divider, InputNumber, Radio, RadioGroup, Select, Switch, Toast } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

import PureInput from '@/components/PureInput'
import { ScrollArea } from '@/components/ScrollArea'
import { useSettings } from '@/context/Settings'
import { useTheme, type ThemeMode } from '@/context/Theme'
import { LOCALES, useLocale, useT, type LocalePreference } from '@/i18n'
import { useCurrentModal } from '@/lib/multi-modal'
import {
  border_box,
  flex,
  flex_1,
  flex_col,
  gap_x_2,
  gap_y_1,
  gap_y_2,
  gap_y_3,
  items_center,
  min_h_0,
} from '@/styles/Classes'
import Logger from '@/utils/Logger'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export type ProxyType = 'http' | 'https' | 'socks'

export interface SettingsValues {
  defaultUsername: string
  defaultPassword: string
  proxyEnabled: boolean
  proxyType: ProxyType
  proxyHost: string
  proxyPort: number | undefined
  proxyUsername: string
  proxyPassword: string
}

const section_title = css`
  font-size: 13px;
  font-weight: 600;
  color: var(--semi-color-text-0);
`

const section_hint = css`
  font-size: 12px;
  color: var(--semi-color-text-2);
`

const port_column = css`
  width: 140px;
`

export function NiceSettingsDialog() {
  const modal = useCurrentModal()

  return (
    <SettingsDialog
      visible={modal.visible}
      afterClose={modal.remove}
      onOk={(values) => {
        // TODO: 保存设置（持久化逻辑后续实现）
        modal.resolve(values)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(undefined)
        modal.hide()
      }}
    ></SettingsDialog>
  )
}

export interface SettingsDialogProps {
  visible: boolean
  afterClose?: () => void
  onOk?: (values: SettingsValues) => void
  onCancel?: () => void
}

export default function SettingsDialog(props: SettingsDialogProps) {
  const t = useT()
  const theme = useTheme((state) => state.theme)
  const localePreference = useLocale((state) => state.preference)

  // 主题与语言先暂存在弹窗里，只有点「确定」才真正落到 store
  const [pendingTheme, setPendingTheme] = useState(theme)
  const [pendingLocale, setPendingLocale] = useState(localePreference)

  const settings = useSettings((state) => state.settings)
  const save = useSettings((state) => state.save)

  const [defaultUsername, setDefaultUsername] = useState(settings.defaultUsername)
  const [defaultPassword, setDefaultPassword] = useState(settings.defaultPassword)

  const [proxyEnabled, setProxyEnabled] = useState(settings.proxyEnabled)
  const [proxyType, setProxyType] = useState<ProxyType>(settings.proxyType)
  const [proxyHost, setProxyHost] = useState(settings.proxyHost)
  const [proxyPort, setProxyPort] = useState<number | undefined>(settings.proxyPort ?? undefined)
  const [proxyUsername, setProxyUsername] = useState(settings.proxyUsername)
  const [proxyPassword, setProxyPassword] = useState(settings.proxyPassword)

  const onOk = async () => {
    if (proxyEnabled) {
      if (proxyHost.trim() === '') {
        Toast.error({ content: t('settings.proxy.hostRequired'), stack: true })
        return
      }
      if (proxyPort === undefined || proxyPort < 1 || proxyPort > 65535) {
        Toast.error({ content: t('settings.proxy.portInvalid'), stack: true })
        return
      }
    }

    try {
      // 写回配置文件，同时应用主题与语言；失败则整体不生效
      await save({
        theme: pendingTheme,
        locale: pendingLocale,
        defaultUsername,
        defaultPassword,
        proxyEnabled,
        proxyType,
        proxyHost: proxyHost.trim(),
        proxyPort: proxyPort ?? null,
        proxyUsername,
        proxyPassword,
      })
    } catch (error) {
      Logger.warn('Failed to save settings: ', error)
      Toast.error({ content: t('settings.saveFailed'), stack: true })
      return
    }

    props.onOk?.({
      defaultUsername,
      defaultPassword,
      proxyEnabled,
      proxyType,
      proxyHost: proxyHost.trim(),
      proxyPort,
      proxyUsername,
      proxyPassword,
    })
  }

  return (
    <Dialog
      size="medium"
      title={t('settings.title')}
      visible={props.visible}
      afterClose={props.afterClose}
      onOk={onOk}
      onCancel={props.onCancel}
    >
      <ScrollArea className={cx(flex_1, min_h_0)}>
        <div className={cx(flex, flex_col, gap_y_3, border_box)}>
          <div className={cx(flex, flex_col, gap_y_2)}>
            <div className={cx(flex, flex_col, gap_y_1)}>
              <div className={section_title}>{t('settings.appearance.title')}</div>
              <div className={section_hint}>{t('settings.appearance.hint')}</div>
            </div>
            <DialogFormItem title={t('settings.appearance.theme')}>
              <RadioGroup
                type="button"
                value={pendingTheme}
                onChange={(e) => setPendingTheme(e.target.value as ThemeMode)}
              >
                <Radio value="system">{t('settings.appearance.system')}</Radio>
                <Radio value="light">{t('settings.appearance.light')}</Radio>
                <Radio value="dark">{t('settings.appearance.dark')}</Radio>
              </RadioGroup>
            </DialogFormItem>
          </div>

          <Divider style={{ margin: '4px 0' }}></Divider>

          <div className={cx(flex, flex_col, gap_y_2)}>
            <div className={cx(flex, flex_col, gap_y_1)}>
              <div className={section_title}>{t('settings.language.title')}</div>
              <div className={section_hint}>{t('settings.language.hint')}</div>
            </div>
            <DialogFormItem title={t('settings.language.locale')}>
              <RadioGroup
                type="button"
                value={pendingLocale}
                onChange={(e) => setPendingLocale(e.target.value as LocalePreference)}
              >
                <Radio value="system">{t('settings.language.system')}</Radio>
                {LOCALES.map((item) => (
                  <Radio key={item.value} value={item.value}>
                    {item.label}
                  </Radio>
                ))}
              </RadioGroup>
            </DialogFormItem>
          </div>

          <Divider style={{ margin: '4px 0' }}></Divider>

          <div className={cx(flex, flex_col, gap_y_2)}>
            <div className={cx(flex, flex_col, gap_y_1)}>
              <div className={section_title}>{t('settings.account.title')}</div>
              <div className={section_hint}>{t('settings.account.hint')}</div>
            </div>
            <DialogFormItem title={t('settings.account.username')}>
              <PureInput
                value={defaultUsername}
                onChange={setDefaultUsername}
                placeholder={t('settings.account.emptyHint')}
              ></PureInput>
            </DialogFormItem>
            <DialogFormItem title={t('settings.account.password')}>
              <PureInput
                mode="password"
                value={defaultPassword}
                onChange={setDefaultPassword}
                placeholder={t('settings.account.emptyHint')}
              ></PureInput>
            </DialogFormItem>
          </div>

          <Divider style={{ margin: '4px 0' }}></Divider>

          <div className={cx(flex, flex_col, gap_y_2)}>
            <div className={cx(flex, items_center, gap_x_2)}>
              <div className={section_title}>{t('settings.proxy.title')}</div>
              <div className={flex_1}></div>
              <Switch checked={proxyEnabled} onChange={setProxyEnabled}></Switch>
            </div>
            <div className={section_hint}>{t('settings.proxy.hint')}</div>
            <DialogFormItem title={t('settings.proxy.type')}>
              <Select
                className={cx(flex_1)}
                value={proxyType}
                onChange={(value) => setProxyType(value as ProxyType)}
                disabled={!proxyEnabled}
              >
                <Select.Option value="http">HTTP</Select.Option>
                <Select.Option value="https">HTTPS</Select.Option>
                <Select.Option value="socks">SOCKS5</Select.Option>
              </Select>
            </DialogFormItem>
            <div className={cx(flex, gap_x_2)}>
              <DialogFormItem className={cx(flex_1, min_h_0)} title={t('settings.proxy.host')}>
                <PureInput
                  value={proxyHost}
                  onChange={setProxyHost}
                  placeholder={t('settings.proxy.hostPlaceholder')}
                  disabled={!proxyEnabled}
                ></PureInput>
              </DialogFormItem>
              <DialogFormItem className={port_column} title={t('settings.proxy.port')}>
                <InputNumber
                  value={proxyPort}
                  onChange={(value) => setProxyPort(typeof value === 'number' ? value : undefined)}
                  min={1}
                  max={65535}
                  placeholder="1-65535"
                  disabled={!proxyEnabled}
                  parser={(value) => value.replace(/[^\d]/g, '')}
                  formatter={(value) => `${value}`.replace(/[^\d]/g, '')}
                ></InputNumber>
              </DialogFormItem>
            </div>
            <DialogFormItem title={t('settings.proxy.auth')}>
              <div className={cx(flex, flex_1, gap_x_2)}>
                <PureInput
                  className={cx(flex_1)}
                  value={proxyUsername}
                  onChange={setProxyUsername}
                  placeholder={t('settings.proxy.usernamePlaceholder')}
                  disabled={!proxyEnabled}
                ></PureInput>
                <PureInput
                  className={cx(flex_1)}
                  mode="password"
                  value={proxyPassword}
                  onChange={setProxyPassword}
                  placeholder={t('settings.proxy.passwordPlaceholder')}
                  disabled={!proxyEnabled}
                ></PureInput>
              </div>
            </DialogFormItem>
          </div>
        </div>
      </ScrollArea>
    </Dialog>
  )
}
