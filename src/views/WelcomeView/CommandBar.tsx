import { cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { Wezterm } from '@thesvg/react'
import { Alacritty } from '@thesvg/react'
import { Warp } from '@thesvg/react'
import { useState } from 'react'
import * as uuid from 'uuid'

import { TabContentModel } from '@/App'
import { ContextMenu } from '@/components/ContextMenu'
import { IconTooltipButton } from '@/components/HoverTooltip'
import IconSelect from '@/components/IconSelect'
import { OpenInExternalApplication } from '@/context/Functions'
import { useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { useT } from '@/i18n'
import { useModal } from '@/lib/multi-modal'
import Logger from '@/utils/Logger'

import LocateFixedIcon from '../../icons/LocateFixed.svg?react'
import OperationCheckoutIcon from '../../icons/OperationCheckout.svg?react'
import OperationExportIcon from '../../icons/OperationExport.svg?react'
import OperationImportIcon from '../../icons/OperationImport.svg?react'
import TerminalIcon from '../../icons/Terminal.svg?react'
import { m_1, gap_x_2, flex, flex_1, items_center, border_radius_5 } from '../../styles/Classes'
import { NiceCheckoutDialog } from '../dialogs/CheckoutDialog'
import { NiceExportDialog } from '../dialogs/ExportDialog'
import { NiceImportDialog } from '../dialogs/ImportDialog'
import { OpenFromPathDialog } from '../dialogs/OpenFromPathDialog'

export function CommandBar() {
  const t = useT()
  const tabManager = useTabManager()
  const tabContent = useTabContent()
  const selectedFolder = async () => {
    let selected = await open({
      title: t('shared.action.selectFolder'),
      multiple: false,
      directory: true,
    })
    if (selected === null) {
      return
    }

    selected = selected.replace(/\\/g, '/')
    Logger.info('replace selected path', selected)
    // const uuid = await uuidCreate()
    const identity = uuid.v4()
    const content: TabContentModel = {
      workspaceView: {
        from: tabContent.identity,
        path: selected,
      },
    }
    tabManager.add(
      {
        content,
        title: t('app.tab.workspace'),
        identity,
      },
      true,
    )
  }

  // const checkoutDialog = useRef<DialogBase>(null)

  // const [checkoutDialog, setCheckoutDialog] = useState(0)

  // const [checkoutDialogVisible, setCheckoutDialogVisible] = useState(false)
  // const [create, setCreate] = useState(false)

  // const [checkoutDialogKey, setCheckoutDialogKey] = useState(0)

  const modal = useModal()
  const onCheckout = () => {
    modal.show(NiceCheckoutDialog, {})
    // setCheckoutDialogKey((i) => i + 1)
    // setCheckoutDialogVisible(true)
    // setCreate(true)
    // setCheckoutDialog((v) => v + 1)
    // checkoutDialog.current?.show()
  }

  const onExport = () => {
    modal.show(NiceExportDialog, {})
  }

  const onImport = () => {
    modal.show(NiceImportDialog, {})
  }

  const onOpenFromPath = () => {
    modal.show(OpenFromPathDialog, {})
  }

  const folderContextMenu = [
    {
      item: {
        content: t('welcome.commandBar.openFromPath'),
        onSelect: () => onOpenFromPath(),
      },
    },
  ]

  interface Item {
    value: string
    label: string
    icon?: React.ReactNode
    onClick?: () => void
  }

  const openItems: Item[] = [
    {
      value: 'Terminal',
      label: 'Terminal',
      icon: <TerminalIcon></TerminalIcon>,
      onClick() {
        OpenInExternalApplication('terminal')
      },
    },
    {
      value: 'WezTerm',
      label: 'WezTerm',
      icon: <Wezterm></Wezterm>,
      onClick() {
        OpenInExternalApplication('wezTerm')
      },
    },
    {
      value: 'Alacritty',
      label: 'Alacritty',
      icon: <Alacritty></Alacritty>,
      onClick() {
        OpenInExternalApplication('alacritty')
      },
    },
    {
      value: 'Warp',
      label: 'Warp',
      icon: <Warp className={cx(border_radius_5)}></Warp>,
      onClick() {
        OpenInExternalApplication('warp')
      },
    },
  ]

  const [openValue, setOpenValue] = useState('Terminal')

  const onSelect = () => {
    Logger.info('open external app: ', openValue)
    switch (openValue) {
      case 'Terminal':
        OpenInExternalApplication('terminal')
        break
      case 'WezTerm':
        OpenInExternalApplication('wezTerm')
        break
      case 'Alacritty':
        OpenInExternalApplication('alacritty')
        break
      case 'Warp':
        OpenInExternalApplication('warp')
        break
    }
  }

  return (
    <div className={cx(m_1, flex, gap_x_2, items_center)}>
      <IconTooltipButton tooltipConent={t('shared.action.checkout')} onClick={onCheckout}>
        <OperationCheckoutIcon></OperationCheckoutIcon>
      </IconTooltipButton>
      <ContextMenu asChild menu={folderContextMenu}>
        <IconTooltipButton
          tooltipConent={t('shared.action.selectWorkingCopy')}
          onClick={selectedFolder}
        >
          <LocateFixedIcon></LocateFixedIcon>
        </IconTooltipButton>
      </ContextMenu>
      <IconTooltipButton tooltipConent={t('shared.action.export')} onClick={onExport}>
        <OperationExportIcon />
      </IconTooltipButton>
      <IconTooltipButton tooltipConent={t('shared.action.import')} onClick={onImport}>
        <OperationImportIcon />
      </IconTooltipButton>
      <div className={cx(flex_1)}></div>
      <IconSelect
        onSelect={onSelect}
        value={openValue}
        onChange={setOpenValue}
        items={openItems}
      ></IconSelect>
    </div>
  )
}
