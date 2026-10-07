import { IconChevronDown, IconChevronUp } from '@douyinfe/semi-icons'
import { Checkbox, Collapsible, Descriptions } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useRef, useState } from 'react'

import { TrustServer } from '@/bindings/TrustServer'
import { ScrollArea } from '@/components/ScrollArea'
import { replySuccess } from '@/context/Functions'
import { SubversionEventMap } from '@/context/Subversion'
import { useT } from '@/i18n'
import {
  border_box,
  break_all,
  break_word,
  flex,
  flex_col,
  gap_y_1,
  hidden,
  items_center,
} from '@/styles/Classes'

import { Dialog } from './Dialog'

const cert_pre = css`
  margin: 0;
  padding: 8px;
  border-radius: 8px;
  border: 1px solid var(--semi-color-border);
  background: var(--semi-color-fill-0);
`

type SslServerTrustPromptDialogProps = SubversionEventMap['sslServerTrustPrompt'] & {
  onClose?: () => void
  afterClose?: () => void
}

export default function SslServerTrustPromptDialog(props: SslServerTrustPromptDialogProps) {
  const t = useT()
  const [visible, setVisible] = useState(true)
  const [save, setSave] = useState(false)
  const [certExpanded, setCertExpanded] = useState(false)

  let expandIcon = certExpanded ? (
    <IconChevronDown onClick={() => setCertExpanded((v) => !v)}></IconChevronDown>
  ) : (
    <IconChevronUp onClick={() => setCertExpanded((v) => !v)}></IconChevronUp>
  )

  expandIcon = <div className={cx(flex, items_center)}>{expandIcon}</div>

  const descriptions = [
    { key: t('shared.field.realm'), value: props.realm },
    { key: t('advancedDialogs.sslTrust.hostname'), value: props.info.hostname },
    { key: t('advancedDialogs.sslTrust.fingerprint'), value: props.info.fingerprint },
    { key: t('advancedDialogs.sslTrust.validFrom'), value: props.info.validFrom },
    { key: t('advancedDialogs.sslTrust.validUntil'), value: props.info.validUntil },
    { key: t('advancedDialogs.sslTrust.issuer'), value: props.info.issuer },
    { key: t('advancedDialogs.sslTrust.asciiCert'), value: expandIcon },
  ]

  const close = () => {
    setVisible(false)
    props.onClose?.()
  }

  const onOk = async () => {
    const value: TrustServer = {
      acceptFailures: props.failures,
      save,
    }
    // const msg: ReplyMessage = {
    //   success: value,
    // }
    await replySuccess(props.id, value)
    close()
  }
  const onCancel = async () => {
    // const msg: ReplyMessage = {
    //   success: null,
    // }
    await replySuccess(props.id, null)
    close()
  }

  const scrollArea = useRef<HTMLDivElement>(null)

  const [padding, setPadding] = useState(false)

  return (
    <Dialog
      afterClose={props.afterClose}
      size="medium"
      visible={visible}
      onOk={onOk}
      onCancel={onCancel}
    >
      <ScrollArea
        ref={scrollArea}
        className={cx(
          flex,
          flex_col,
          border_box,
          padding &&
            css`
              margin-bottom: 1px;
            `,
        )}
        contentClassName={cx(flex, flex_col, gap_y_1)}
      >
        <Descriptions data={descriptions}></Descriptions>
        <Collapsible
          onMotionEnd={() => {
            requestAnimationFrame(() => {
              setPadding((v) => !v)
            })
          }}
          isOpen={certExpanded}
        >
          <div className={cx(cert_pre, break_all, break_word)}>{props.info.asciiCert}</div>
        </Collapsible>
        <Checkbox
          checked={save}
          onChange={(e) => setSave(e.target.checked ?? false)}
          className={cx(!props.maySave && hidden)}
        >
          {t('shared.action.save')}
        </Checkbox>
      </ScrollArea>
    </Dialog>
  )
}
