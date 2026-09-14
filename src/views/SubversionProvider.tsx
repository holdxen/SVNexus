import { cx } from '@linaria/core'
import React, { useEffect, useMemo, useState } from 'react'

import {
  createSingletonSubversion,
  createTransientSubversion,
  SubverionContext,
  Subversion,
  SubversionEventMap,
} from '@/context/Subversion'
import { useModal } from '@/lib/multi-modal'
import { flex, flex_1, min_w_0 } from '@/styles/Classes'

import AuthenticateDialog from './dialogs/AuthenticateDialog'
import { NiceConflictDialog } from './dialogs/ConflictDialog'
import MaySavePasswordAsPlainText from './dialogs/MaySavePasswordAsPlainText'
import { NiceSslClientCertificateDialog } from './dialogs/SslClientCertificateDialog'
import SslServerTrustPromptDialog from './dialogs/SslServerTrustPromptDialog'

export function SubversionProvider({
  children,
  singleton,
  className,
}: {
  children?: React.ReactNode
  singleton: boolean
  className?: string
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

  const factory = useMemo(() => {
    const onCreated = (subversion: Subversion) => {
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
    }

    return singleton ? createSingletonSubversion(onCreated) : createTransientSubversion(onCreated)
  }, [])

  useEffect(() => {
    return () => {
      factory.releaseAll()
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
