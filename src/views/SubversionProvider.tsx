import { cx } from '@linaria/core'
import React, { useEffect, useMemo, useRef, useState } from 'react'

import { useNotifyLogStore } from '@/context/NotifyLog'
import {
  createSingletonSubversion,
  createTransientSubversion,
  SubverionContext,
  Subversion,
  SubversionEventMap,
} from '@/context/Subversion'
import type { Identity } from '@/context/TabContent'
import { useModal } from '@/lib/multi-modal'
import { flex, flex_1, min_w_0 } from '@/styles/Classes'
import Logger from '@/utils/Logger'

import AuthenticateDialog from './dialogs/AuthenticateDialog'
import { NiceConflictDialog } from './dialogs/ConflictDialog'
import MaySavePasswordAsPlainText from './dialogs/MaySavePasswordAsPlainText'
import { NiceSSHAuthenticateDialog } from './dialogs/SSHAuthenticateDialog'
import { NiceSSHKeyboardInteractiveDialog } from './dialogs/SSHKeyboardInteractiveDialog'
import { NiceSSHPassphraseDialog } from './dialogs/SSHPassphraseDialog'
import { NiceSSHVerifyNewHostKeyDialog } from './dialogs/SSHVerifyNewHostKeyDialog'
import { NiceSslClientCertificateDialog } from './dialogs/SslClientCertificateDialog'
import SslServerTrustPromptDialog from './dialogs/SslServerTrustPromptDialog'
import { useSettings } from '@/context/Settings'
import { useMemoizedFn } from 'ahooks'

export function SubversionProvider({
  children,
  singleton,
  className,
  notifyLogIdentity,
}: {
  children?: React.ReactNode
  singleton: boolean
  className?: string
  /**
   * 传入后，这个 provider 下所有操作产生的 workingCopyNotify 都会记进对应 tab 的通知日志。
   * 订阅必须在这里做：onCreated 与上下文创建同步执行，早于任何命令，不会漏掉第一个操作的通知。
   */
  notifyLogIdentity?: Identity
}) {
  // const authenticateDialogs:  = useMemo(() => [], [])
  const [authenticateDialogs, setAuthenticateDialogs] = useState<
    SubversionEventMap['authenticate'][]
  >([])

  const [sslServerTrustPromptDialogs, setSslServerTrustPromptDialogs] = useState<
    SubversionEventMap['sslServerTrustPrompt'][]
  >([])

  const [maySavePasswordAsPlainTextDialogs, setMaySavePasswordAsPlainTextDialogs] = useState<
    SubversionEventMap['savePasswordAsPlainText'][]
  >([])

  const modal = useModal()

  // factory 只在挂载时构建一次，用 ref 读取当前 identity，避免把它写进 useMemo 依赖导致重建单例
  const notifyLogIdentityRef = useRef(notifyLogIdentity)
  notifyLogIdentityRef.current = notifyLogIdentity

  const settings = useSettings()

  const onCreating = useMemoizedFn(() => {
    return settings.settings
  })

  const factory = useMemo(() => {
    const onCreated = (subversion: Subversion) => {
      const logIdentity = notifyLogIdentityRef.current
      if (logIdentity !== undefined) {
        subversion.on('workingCopyNotify', (data) => {
          useNotifyLogStore.getState().append(logIdentity, data.notify)
        })
      }
      subversion.on('authenticate', (data) => {
        setAuthenticateDialogs((items) => [...items, data])
      })
      subversion.on('sslServerTrustPrompt', (data) => {
        setSslServerTrustPromptDialogs((items) => [...items, data])
      })
      subversion.on('savePasswordAsPlainText', (data) => {
        setMaySavePasswordAsPlainTextDialogs((items) => [...items, data])
      })
      subversion.on('conflict', (data) => {
        modal.show(NiceConflictDialog, {
          ...data,
        })
      })
      subversion.on('sslClientCertificate', (data) => {
        modal.show(NiceSslClientCertificateDialog, data)
      })
      subversion.on('tunnel', (data) => {
        Logger.info('On get tunnel event: ', data.ssh)
        if ('authenticate' in data.ssh) {
          modal.show(NiceSSHAuthenticateDialog, data.ssh.authenticate)
        } else if ('verifyNewHostKey' in data.ssh) {
          modal.show(NiceSSHVerifyNewHostKeyDialog, data.ssh.verifyNewHostKey)
        } else if ('keyboardInteractive' in data.ssh) {
          modal.show(NiceSSHKeyboardInteractiveDialog, data.ssh.keyboardInteractive)
        } else if ('passphrase' in data.ssh) {
          modal.show(NiceSSHPassphraseDialog, data.ssh.passphrase)
        }
      })
    }


    return singleton ? createSingletonSubversion(onCreated, onCreating) : createTransientSubversion(onCreated, onCreating)
  }, [])

  useEffect(() => {
    return () => {
      factory.releaseAll()
      if (notifyLogIdentityRef.current !== undefined) {
        useNotifyLogStore.getState().remove(notifyLogIdentityRef.current)
      }
    }
  }, [])

  return (
    <div className={cx(flex, className)}>
      <SubverionContext.Provider value={factory}>
        <div className={cx(flex_1, flex, min_w_0)}>{children}</div>
        <div>
          {authenticateDialogs.map((item) => {
            return (
              <AuthenticateDialog
                {...item}
                key={item.id.toString()}
                afterClose={() => {
                  setAuthenticateDialogs((items) => items.filter((i) => i.id !== item.id))
                }}
              ></AuthenticateDialog>
            )
          })}
          {sslServerTrustPromptDialogs.map((item) => {
            return (
              <SslServerTrustPromptDialog
                {...item}
                key={item.id.toString()}
                afterClose={() => {
                  setSslServerTrustPromptDialogs((items) => items.filter((i) => i.id !== item.id))
                }}
              ></SslServerTrustPromptDialog>
            )
          })}
          {maySavePasswordAsPlainTextDialogs.map((item) => {
            return (
              <MaySavePasswordAsPlainText
                {...item}
                key={item.id.toString()}
                afterClose={() => {
                  setMaySavePasswordAsPlainTextDialogs((items) =>
                    items.filter((i) => i.id !== item.id),
                  )
                }}
              ></MaySavePasswordAsPlainText>
            )
          })}
        </div>
      </SubverionContext.Provider>
    </div>
  )
}
