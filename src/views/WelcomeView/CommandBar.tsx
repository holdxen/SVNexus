import { cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import * as uuid from 'uuid'

import { ContextMenu } from '@/components/ContextMenu'
import { useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { useModal } from '@/lib/multi-modal'

import { IconButton } from '../../icons/IconButton'
import LocateFixedIcon from '../../icons/LocateFixed.svg?react'
import OperationCheckoutIcon from '../../icons/OperationCheckout.svg?react'
import OperationExportIcon from '../../icons/OperationExport.svg?react'
import TerminalIcon from '../../icons/Terminal.svg?react'
import { m_1, gap_x_2, flex, flex_1 } from '../../styles/Classes'
import { NiceCheckoutDialog } from '../dialogs/CheckoutDialog'
import { NiceExportDialog } from '../dialogs/ExportDialog'
import { OpenFromPathDialog } from '../dialogs/OpenFromPathDialog'
import { TabContentModel } from '@/App'

export function CommandBar() {
  const tabManager = useTabManager()
  const tabContent = useTabContent()
  const selectedFolder = async () => {
    const selected = await open({
      title: 'Select folder',
      multiple: false,
      directory: true,
    })
    if (selected === null) {
      return
    }
    // const uuid = await uuidCreate()
    const identity = uuid.v4()
    const content: TabContentModel = {
      'workspaceView': {
        from: tabContent.identity,
        path: selected
      }
    }
    tabManager.add(
      {
        content,
        title: 'Workspace',
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

  const onOpenFromPath = () => {
    modal.show(OpenFromPathDialog, {})
  }

  const folderContextMenu = [
    {
      item: {
        content: '从路径打开',
        onSelect: () => onOpenFromPath(),
      },
    },
  ]

  return (
    <div className={cx(m_1, flex, gap_x_2)}>
      <IconButton onClick={onCheckout}>
        <OperationCheckoutIcon></OperationCheckoutIcon>
      </IconButton>
      <ContextMenu asChild menu={folderContextMenu}>
        <IconButton onClick={selectedFolder}>
          <LocateFixedIcon></LocateFixedIcon>
        </IconButton>
      </ContextMenu>
      <IconButton onClick={onExport}>
        <OperationExportIcon />
      </IconButton>
      <div className={cx(flex_1)}>
        {/*{checkoutDialogKey !== 0 ? (
          <CheckoutDialog key={checkoutDialogKey}></CheckoutDialog>
        ) : (
          <></>
        )}*/}
      </div>
      <IconButton>
        <TerminalIcon></TerminalIcon>
      </IconButton>
    </div>
  )
}
