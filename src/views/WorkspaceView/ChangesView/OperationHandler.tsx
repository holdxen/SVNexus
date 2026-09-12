import { cx } from '@linaria/core'

import { InfoEntry } from '@/bindings/InfoEntry'
import { MergeSource } from '@/bindings/MergeSource'
import { formatSize } from '@/context/Functions'
import { useModal } from '@/lib/multi-modal'
import { select_text } from '@/styles/Classes'
import { NiceAddDialog } from '@/views/dialogs/AddDialog'
import { NiceCommitDialog } from '@/views/dialogs/CommitDialog'
import { NiceDeleteDialog } from '@/views/dialogs/DeleteDialog'
import { NiceDifferenceDialog } from '@/views/dialogs/DifferenceDialog'
import { NiceExportDialog } from '@/views/dialogs/ExportDialog'
import { NiceInfoDialog } from '@/views/dialogs/InfoDialog'
import { NiceLockDialog } from '@/views/dialogs/LockDialog'
import { NiceMergeDialog } from '@/views/dialogs/MergeDialog'
import { NiceMkdirDialog } from '@/views/dialogs/MkdirDialog'
import { NicePatchDialog } from '@/views/dialogs/PatchDialog'
import { NiceRevertDialog } from '@/views/dialogs/RevertDialog'
import { NiceSwitchDialog } from '@/views/dialogs/SwitchDialog'
import { NiceUnlockDialog } from '@/views/dialogs/UnlockDialog'
import { NiceUpdateDialog } from '@/views/dialogs/UpdateDialog'

import { WorkingCopyPathItemModel } from '../WorkingCopyItem'

export default class OperationHandler {
  private modal: ReturnType<typeof useModal>
  private refresh: () => Promise<void>
  constructor(modal: ReturnType<typeof useModal>, refresh: () => Promise<void>) {
    this.modal = modal
    this.refresh = refresh
  }

  public async showAddDialog(items: WorkingCopyPathItemModel[]) {
    const result = await this.modal.show(NiceAddDialog, { items }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showUpdateDialog(items: WorkingCopyPathItemModel[]) {
    const result = await this.modal.show(NiceUpdateDialog, { items }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showRevertDialog(items: WorkingCopyPathItemModel[]) {
    const result = await this.modal.show(NiceRevertDialog, { items }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public showDifferenceDialog(workingCopy: string, workspace: string, path: string) {
    this.modal.show(NiceDifferenceDialog, {
      workingCopy,
      workspace,
      source: {
        target: {
          path1: path,
          revision1: 'base',
          path2: path,
          revision2: 'working',
        },
      },
    })
  }

  public showPatchDialog(patchFile: string, workingCopyPath: string) {
    this.modal.show(NicePatchDialog, { patchFile, workingCopyPath })
  }

  public async showLockDialog(items: WorkingCopyPathItemModel[]) {
    const result = await this.modal.show(NiceLockDialog, { items }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showUnlockDialog(items: WorkingCopyPathItemModel[]) {
    const result = await this.modal.show(NiceUnlockDialog, { items }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showCommitDialog(items: WorkingCopyPathItemModel[], relateTo?: string) {
    const result = await this.modal.show(NiceCommitDialog, { items, relateTo }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showDeleteDialog(items: WorkingCopyPathItemModel[], needCommitMessage = false) {
    const result = await this.modal
      .show(NiceDeleteDialog, { items, needCommitMessage })
      .as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showMkdirDialog(parent: string, needCommitMessage = false) {
    const result = await this.modal
      .show(NiceMkdirDialog, { parent, needCommitMessage })
      .as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showSwitchDialog(path: string, url?: string) {
    const result = await this.modal.show(NiceSwitchDialog, { path, url }).as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public async showMergeDialog(defaultTarget?: string, defaultSource?: MergeSource) {
    const result = await this.modal
      .show(NiceMergeDialog, { defaultTarget, defaultSource })
      .as<boolean>()
    if (result) {
      await this.refresh()
    }
  }

  public showExportDialog(defaultPath?: string) {
    this.modal.show(NiceExportDialog, { defaultPath })
  }

  public async showInfoEntryDialog(info: InfoEntry) {
    const modal = this.modal
    const size = info.size ?? info.workingCopyInfo?.recordedSize
    const sizeText = size ? await formatSize(size) : null
    await modal
      .show(NiceInfoDialog, {
        data: [
          {
            key: 'Url:',
            value: <div className={cx(select_text)}>{info.url}</div>,
          },
          { key: 'Kind:', value: <div className={cx(select_text)}>{info.kind}</div> },
          {
            key: 'Size:',
            value: <div className={cx(select_text)}>{sizeText}</div>,
            hidden: sizeText === null,
          },
          {
            key: 'Revision:',
            value: <div className={cx(select_text)}>{info.revision?.toString() ?? null}</div>,
          },
          {
            key: 'Repository UUID:',
            value: <div className={cx(select_text)}>{info.repositoryUuid}</div>,
          },
          {
            key: 'Repository root URL:',
            value: <div className={cx(select_text)}>{info.repositoryRootUrl}</div>,
          },
          {
            key: 'Changelist:',
            value: <div className={cx(select_text)}>{info.workingCopyInfo?.changelist}</div>,
            hidden: info.workingCopyInfo?.changelist === null,
          },
          {
            key: 'From:',
            value: (
              <div className={cx(select_text)}>{info.workingCopyInfo?.movedFromAbsolutePath}</div>
            ),
            hidden: info.workingCopyInfo?.movedFromAbsolutePath === null,
          },
          {
            key: 'To:',
            value: (
              <div className={cx(select_text)}>{info.workingCopyInfo?.movedToAbsolutePath}</div>
            ),
            hidden: info.workingCopyInfo?.movedToAbsolutePath === null,
          },
          {
            key: 'WC format:',
            value: <div className={cx(select_text)}>{info.workingCopyInfo?.workingCopyFormat}</div>,
            hidden: info.workingCopyInfo === null,
          },
          {
            key: 'Lock path:',
            value: <div className={cx(select_text)}>{info.lock?.path}</div>,
            hidden: typeof info.lock?.path !== 'string',
          },
          {
            key: 'Lock owner:',
            value: <div className={cx(select_text)}>{info.lock?.owner}</div>,
            hidden: info.lock === null,
          },
          {
            key: 'Lock create date:',
            value: (
              <div className={cx(select_text)}>
                {info.lock?.creationDate ? new Date(info.lock.creationDate).toString() : ''}
              </div>
            ),
            hidden: info.lock === null,
          },
          {
            key: 'Lock expire date:',
            value: (
              <div className={cx(select_text)}>
                {info.lock?.expirationDate ? new Date(info.lock.expirationDate).toString() : ''}
              </div>
            ),
            hidden: info.lock === null,
          },
        ],
      })
      .as<unknown>()
  }
}
