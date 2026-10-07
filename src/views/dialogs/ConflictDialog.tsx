import { AutoComplete, Tag } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

import { WorkingCopyConflictChoice } from '@/bindings/WorkingCopyConflictChoice'
import { WorkingCopyConflictDescription } from '@/bindings/WorkingCopyConflictDescription'
import { WorkingCopyConflictResult } from '@/bindings/WorkingCopyConflictResult'
import { WorkingCopyConflictVersion } from '@/bindings/WorkingCopyConflictVersion'
import { ScrollArea } from '@/components/ScrollArea'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import { replySuccess } from '@/context/Functions'
import { useT } from '@/i18n'
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
  grid_cols_1fr_1fr,
  p_1,
  text_12px,
} from '@/styles/Classes'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

export interface ConflictDialogProps {
  id: number
  description: WorkingCopyConflictDescription
  visible: boolean
  afterClose?: () => void
  onOk?: () => void
  onCancel?: () => void
}

// ==================== Styles ====================

const badge = css`
  display: inline-flex;
  align-items: center;
  justify-content: center;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  letter-spacing: 0.5px;
  text-transform: uppercase;
  flex-shrink: 0;
`

// 文字取色阶 6、底色取色阶 0：Semi 在暗色下会整体翻转色阶，两级都能保住对比度
const badge_text = css`
  background: rgba(var(--semi-light-blue-0), 1);
  color: rgba(var(--semi-light-blue-6), 1);
`

const badge_property = css`
  background: rgba(var(--semi-orange-0), 1);
  color: rgba(var(--semi-orange-6), 1);
`

const badge_tree = css`
  background: rgba(var(--semi-purple-0), 1);
  color: rgba(var(--semi-purple-6), 1);
`

const summary_row = css`
  display: flex;
  align-items: center;
  gap: 8px;
  flex-wrap: wrap;
`

const path_text = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 12px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  max-width: 400px;
`

const section_title = css`
  font-size: 12px;
  text-transform: uppercase;
  letter-spacing: 1.2px;
  color: rgba(var(--semi-grey-7), 1);
  margin: 0;
  font-weight: 600;
`

const val_card = css`
  background: var(--semi-color-fill-0);
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
  padding: 10px 12px;
  display: flex;
  flex-direction: column;
  gap: 4px;
`

const val_card_role = css`
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 11px;
  color: rgba(var(--semi-grey-7), 1);
`

const val_card_field = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 10px;
  color: rgba(var(--semi-grey-6), 1);
`

const val_card_code = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 13px;
  font-weight: 600;
  padding: 2px 8px;
  border-radius: 4px;
  background: rgba(var(--semi-grey-8), 0.06);
  word-break: break-all;
`

const val_card_note = css`
  font-size: 11px;
  color: rgba(var(--semi-grey-7), 1);
  line-height: 1.4;
`

const val_base = css`
  border-left: 3px solid rgba(var(--semi-grey-6), 1);
`

const val_working = css`
  border-left: 3px solid rgba(var(--semi-green-5), 1);
`

const val_incoming_old = css`
  border-left: 3px solid rgba(var(--semi-purple-5), 1);
`

const val_incoming_new = css`
  border-left: 3px solid rgba(var(--semi-blue-5), 1);
`

const trap_box = css`
  background: rgba(var(--semi-orange-1), 0.15);
  border: 1px solid rgba(var(--semi-orange-5), 0.4);
  border-radius: 8px;
  padding: 10px 14px;
  font-size: 12px;
  line-height: 1.5;
  color: rgba(var(--semi-grey-8), 1);
`

const options_grid = css`
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 10px;
`

const opt_btn = css`
  text-align: left;
  background: var(--semi-color-fill-0);
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
  padding: 10px 12px;
  cursor: pointer;
  color: var(--semi-color-text-0);
  font: inherit;
  transition:
    border-color 0.15s,
    background 0.15s;

  &:hover {
    border-color: var(--semi-color-primary);
    background: rgba(var(--semi-blue-0), 0.06);
  }

  &:active {
    transform: scale(0.98);
  }
`

const opt_btn_selected = css`
  border-color: var(--semi-color-primary);
  background: rgba(var(--semi-blue-0), 0.14);
`

const opt_name = css`
  font-weight: 600;
  font-size: 13px;
  display: flex;
  align-items: center;
  gap: 6px;
`

const opt_flag = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 10px;
  color: rgba(var(--semi-grey-7), 1);
  font-weight: 500;
`

const opt_desc = css`
  font-size: 11px;
  color: rgba(var(--semi-grey-7), 1);
  margin-top: 4px;
  line-height: 1.4;
`

const opt_result = css`
  margin-top: 6px;
  font-size: 11px;
  color: rgba(var(--semi-grey-7), 1);

  code {
    font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
    font-weight: 600;
    padding: 1px 6px;
    border-radius: 4px;
  }
`

const file_card = css`
  background: var(--semi-color-fill-0);
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
`

const file_card_label = css`
  font-size: 11px;
  font-weight: 600;
  color: rgba(var(--semi-grey-7), 1);
`

const file_card_path = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 11px;
  color: rgba(var(--semi-grey-8), 1);
  word-break: break-all;
`

const binary_banner = css`
  background: rgba(var(--semi-orange-1), 0.15);
  border: 1px solid rgba(var(--semi-orange-5), 0.4);
  border-radius: 8px;
  padding: 8px 14px;
  font-size: 12px;
  display: flex;
  align-items: center;
  gap: 8px;
`

const tree_summary = css`
  display: flex;
  flex-direction: column;
  gap: 6px;
  padding: 10px 14px;
  background: var(--semi-color-fill-0);
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
`

const tree_row = css`
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
`

const tree_row_label = css`
  font-weight: 600;
  min-width: 40px;
  color: rgba(var(--semi-grey-7), 1);
`

const version_card = css`
  background: var(--semi-color-fill-0);
  border: 1px solid var(--semi-color-border);
  border-radius: 8px;
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 2px;
`

const version_card_label = css`
  font-size: 11px;
  font-weight: 600;
  color: rgba(var(--semi-grey-7), 1);
`

const version_card_detail = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
  font-size: 11px;
  color: rgba(var(--semi-grey-8), 1);
`

const mono = css`
  font-family: 'SFMono-Regular', Consolas, 'Liberation Mono', Menlo, monospace;
`

// ==================== Helpers ====================

type Translate = ReturnType<typeof useT>

function kindBadge(kind: string) {
  switch (kind) {
    case 'text':
      return <span className={cx(badge, badge_text)}>TEXT</span>
    case 'property':
      return <span className={cx(badge, badge_property)}>PROP</span>
    case 'tree':
      return <span className={cx(badge, badge_tree)}>TREE</span>
    default:
      return <span className={cx(badge)}>{kind.toUpperCase()}</span>
  }
}

function actionLabel(action: string, t: Translate): string {
  switch (action) {
    case 'edit':
      return t('conflict.action.edit')
    case 'add':
      return t('conflict.action.add')
    case 'delete':
      return t('conflict.action.delete')
    case 'replace':
      return t('conflict.action.replace')
    default:
      return action
  }
}

function reasonLabel(reason: string, t: Translate): string {
  switch (reason) {
    case 'edited':
      return t('conflict.reason.edited')
    case 'obstructed':
      return t('conflict.reason.obstructed')
    case 'deleted':
      return t('conflict.reason.deleted')
    case 'missing':
      return t('conflict.reason.missing')
    case 'unversioned':
      return t('conflict.reason.unversioned')
    case 'added':
      return t('conflict.reason.added')
    case 'replaced':
      return t('conflict.reason.replaced')
    case 'movedAway':
      return t('conflict.reason.movedAway')
    case 'movedHere':
      return t('conflict.reason.movedHere')
    default:
      return reason
  }
}

function operationLabel(op: string, t: Translate): string {
  switch (op) {
    case 'update':
      return t('conflict.operation.update')
    case 'switch':
      return t('conflict.operation.switch')
    case 'merge':
      return t('conflict.operation.merge')
    case 'none':
      return ''
    default:
      return op
  }
}

function nodeKindLabel(nk: string, t: Translate): string {
  switch (nk) {
    case 'file':
      return t('conflict.nodeKind.file')
    case 'directory':
      return t('conflict.nodeKind.directory')
    case 'symlink':
      return t('conflict.nodeKind.symlink')
    case 'none':
      return t('conflict.nodeKind.none')
    default:
      return nk
  }
}

function versionSummary(v: WorkingCopyConflictVersion | null, t: Translate): string {
  if (!v) return t('conflict.value.unavailable')
  const rev = v.pegRevision
  const path = v.pathInRepository
  return `${path}@${rev} (${nodeKindLabel(v.nodeKind, t)})`
}

function displayValue(val: string | null, t: Translate): string {
  if (val === null || val === undefined) return t('conflict.value.unset')
  if (val === '') return t('conflict.value.empty')
  return `"${val}"`
}

// ==================== Sub-components ====================

function Header({ description }: { description: WorkingCopyConflictDescription }) {
  const t = useT()
  const op = operationLabel(description.operation, t)
  const summary = t('conflict.summary.pair', {
    action: actionLabel(description.action, t),
    reason: reasonLabel(description.reason, t),
  })

  return (
    <div className={cx(flex_col, gap_y_1)}>
      <div className={summary_row}>
        {kindBadge(description.kind)}
        <FileKindIcon kind={description.nodeKind} />
        <span className={path_text}>{description.localAbsolutePath}</span>
        {op && (
          <>
            <span style={{ color: 'rgba(var(--semi-grey-6), 1)' }}>·</span>
            <Tag size="small" color="blue">
              {op}
            </Tag>
          </>
        )}
      </div>
      <div className={cx(text_12px)} style={{ color: 'rgba(var(--semi-grey-7), 1)' }}>
        {summary}
      </div>
    </div>
  )
}

function TextBody({ description }: { description: WorkingCopyConflictDescription }) {
  const t = useT()
  const files: { label: string; path: string | null; colorClass: string }[] = [
    { label: t('conflict.role.base'), path: description.baseAbsolutePath, colorClass: val_base },
    { label: t('conflict.role.mine'), path: description.myAbsolutePath, colorClass: val_working },
    {
      label: t('conflict.role.theirs'),
      path: description.theirAbsolutePath,
      colorClass: val_incoming_new,
    },
    {
      label: t('conflict.role.merged'),
      path: description.mergedFile,
      colorClass: val_incoming_old,
    },
  ]

  return (
    <div className={cx(flex_col, gap_y_2)}>
      {description.isBinary && (
        <div className={binary_banner}>
          <span>⚠</span>
          <span>
            {t('conflict.text.binaryPrefix')}
            <code className={mono}>{description.mimeType ?? t('conflict.text.mimeUnknown')}</code>
            {t('conflict.text.binarySuffix')}
          </span>
        </div>
      )}
      <h3 className={section_title}>{t('conflict.text.filesTitle')}</h3>
      <div className={cx(flex, flex_col, gap_x_2, gap_y_2)}>
        {files.map((f) => (
          <div key={f.label} className={cx(file_card, f.colorClass)}>
            <span className={file_card_label}>{f.label}</span>
            <span className={file_card_path}>{f.path ?? t('conflict.value.unavailable')}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function PropertyBody({ description }: { description: WorkingCopyConflictDescription }) {
  const t = useT()
  const propName = description.propertyName ?? t('components.conflict.unknownProperty')

  // Action summary
  const incomingOld = description.propertyValueIncomingOld
  const incomingNew = description.propertyValueIncomingNew
  const working = description.propertyValueWorking

  const incomingSummary =
    description.action === 'edit'
      ? t('conflict.property.summaryEdit', {
          name: propName,
          old: displayValue(incomingOld, t),
          new: displayValue(incomingNew, t),
        })
      : description.action === 'add'
        ? t('conflict.property.summaryAdd', {
            name: propName,
            value: displayValue(incomingNew, t),
          })
        : description.action === 'delete'
          ? t('conflict.property.summaryDelete', {
              name: propName,
              value: displayValue(incomingOld, t),
            })
          : t('conflict.property.summaryOther', { action: description.action, name: propName })

  const localSummary =
    description.reason === 'edited'
      ? t('conflict.property.localEdited', { value: displayValue(working, t) })
      : description.reason === 'added'
        ? t('conflict.property.localAdded', { value: displayValue(working, t) })
        : reasonLabel(description.reason, t)

  const values = [
    {
      role: t('conflict.role.base'),
      field: 'propertyValueBase',
      value: description.propertyValueBase,
      cls: val_base,
      note:
        description.operation === 'merge'
          ? t('conflict.property.noteBaseMerge')
          : t('conflict.property.noteBaseUpdate'),
    },
    {
      role: t('conflict.role.working'),
      field: 'propertyValueWorking',
      value: description.propertyValueWorking,
      cls: val_working,
      note: t('conflict.property.noteWorking'),
    },
    {
      role: t('conflict.role.incomingOld'),
      field: 'propertyValueIncomingOld',
      value: description.propertyValueIncomingOld,
      cls: val_incoming_old,
      note: t('conflict.property.noteIncomingOld'),
    },
    {
      role: t('conflict.role.incomingNew'),
      field: 'propertyValueIncomingNew',
      value: description.propertyValueIncomingNew,
      cls: val_incoming_new,
      note: t('conflict.property.noteIncomingNew'),
    },
  ]

  return (
    <div className={cx(flex_col, gap_y_2)}>
      <h3 className={section_title}>
        {t('conflict.property.title')} <code className={mono}>"{propName}"</code>
      </h3>
      <div className={cx(flex_col, gap_y_1, p_1)}>
        <div className={text_12px}>
          <span style={{ color: 'rgba(var(--semi-blue-5), 1)', fontWeight: 600 }}>
            {t('conflict.property.incomingLabel')}
          </span>
          {incomingSummary}
          <span className={opt_flag} style={{ marginLeft: 6 }}>
            {t('components.conflict.actionFlag', { action: description.action })}
          </span>
        </div>
        <div className={text_12px}>
          <span style={{ color: 'rgba(var(--semi-green-5), 1)', fontWeight: 600 }}>
            {t('conflict.property.localLabel')}
          </span>
          {localSummary}
          <span className={opt_flag} style={{ marginLeft: 6 }}>
            {t('components.conflict.reasonFlag', { reason: description.reason })}
          </span>
        </div>
      </div>

      <h3 className={section_title}>{t('conflict.property.valuesTitle')}</h3>
      <div className={cx(grid_cols_1fr_1fr, gap_x_2, gap_y_2)}>
        {values.map((v) => (
          <div key={v.field} className={cx(val_card, v.cls)}>
            <div className={val_card_role}>
              <span>{v.role}</span>
              <span className={val_card_field}>{v.field}</span>
            </div>
            <code className={val_card_code}>{displayValue(v.value, t)}</code>
            <div className={val_card_note}>{v.note}</div>
          </div>
        ))}
      </div>

      <div className={trap_box}>
        <b>{t('conflict.property.trapTitle')}</b>
        {t('conflict.property.trapIntro')}
        <code className={mono}>propertyValueBase</code>
        {t('conflict.property.trapBaseShown')}
        <b>base</b>
        {t('conflict.property.trapBaseApplies')}{' '}
        <code className={mono}>propertyValueIncomingOld</code>
        {t('conflict.property.trapEnd')}
      </div>
    </div>
  )
}

function TreeBody({ description }: { description: WorkingCopyConflictDescription }) {
  const t = useT()
  const localText = t('conflict.tree.localSummary', {
    kind: nodeKindLabel(description.nodeKind, t),
    reason: reasonLabel(description.reason, t),
  })
  const incomingNodeKind =
    description.action === 'edit' || description.action === 'delete'
      ? (description.sourceLeftVersion?.nodeKind ?? 'unknown')
      : (description.sourceRightVersion?.nodeKind ?? 'unknown')
  const incomingText = t('conflict.tree.incomingSummary', {
    kind: nodeKindLabel(incomingNodeKind, t),
    action: actionLabel(description.action, t),
  })

  return (
    <div className={cx(flex_col, gap_y_2)}>
      <h3 className={section_title}>{t('conflict.tree.summaryTitle')}</h3>
      <div className={tree_summary}>
        <div className={tree_row}>
          <span className={tree_row_label}>{t('conflict.tree.localLabel')}</span>
          <span>{localText}</span>
        </div>
        <div className={tree_row}>
          <span className={tree_row_label}>{t('conflict.tree.incomingLabel')}</span>
          <span>{incomingText}</span>
        </div>
        <div className={tree_row}>
          <span className={tree_row_label}>{t('conflict.tree.operationLabel')}</span>
          <span>
            {description.operation !== 'none'
              ? t('components.conflict.uponOperation', { operation: description.operation })
              : t('conflict.tree.noOperation')}
          </span>
        </div>
      </div>

      <h3 className={section_title}>{t('conflict.tree.versionTitle')}</h3>
      <div className={cx(flex, flex_col, gap_x_2, gap_y_1)}>
        <div className={version_card}>
          <span className={version_card_label}>merge-left (src_left_version)</span>
          <span className={version_card_detail}>
            {description.sourceLeftVersion
              ? versionSummary(description.sourceLeftVersion, t)
              : t('conflict.value.legacyUnavailable')}
          </span>
          {description.sourceLeftVersion && (
            <span className={cx(text_12px)} style={{ color: 'rgba(var(--semi-grey-7), 1)' }}>
              {description.sourceLeftVersion.repositoryUrl}
            </span>
          )}
        </div>
        <div className={version_card}>
          <span className={version_card_label}>merge-right (src_right_version)</span>
          <span className={version_card_detail}>
            {description.sourceRightVersion
              ? versionSummary(description.sourceRightVersion, t)
              : t('conflict.value.legacyUnavailable')}
          </span>
          {description.sourceRightVersion && (
            <span className={cx(text_12px)} style={{ color: 'rgba(var(--semi-grey-7), 1)' }}>
              {description.sourceRightVersion.repositoryUrl}
            </span>
          )}
        </div>
      </div>
    </div>
  )
}

// ==================== Choice Buttons ====================

interface ChoiceOption {
  choice: WorkingCopyConflictChoice
  name: string
  flag: string
  desc: string
  resultValue: string
  resultColor: string
}

function getChoiceOptions(
  description: WorkingCopyConflictDescription,
  t: Translate,
): ChoiceOption[] {
  const options: ChoiceOption[] = []

  if (description.kind === 'property') {
    // Property conflict choices accepted via conflict_func2
    // (generate_propconflict in conflicts.c):
    // - postpone: conflict remains (conflicts.c:1358)
    // - mine_full: keep the working value (1363)
    // - theirs_full: adopt the incoming new value (1375)
    // - base: old_props[propname] — the incoming old value for
    //   update/switch, the pristine BASE value for merge (1381, 1942-1946)
    // - merged: requires a merged_value (or merged_file) from the result,
    //   otherwise SVN errors (1389-1394)
    // - mine_conflict/theirs_conflict are NOT handled here — they fall into
    //   the default case (1357) and silently act as postpone, so don't offer
    options.push({
      choice: 'postpone',
      name: t('conflict.choice.postpone'),
      flag: 'postpone',
      desc: t('conflict.choice.postponeDesc'),
      resultValue: t('conflict.choice.resultConflict'),
      resultColor: 'var(--semi-color-tertiary)',
    })
    options.push({
      choice: 'mineFull',
      name: t('conflict.choice.mineFull'),
      flag: 'mine-full',
      desc: t('conflict.choice.mineFullDesc'),
      resultValue: displayValue(description.propertyValueWorking, t),
      resultColor: 'var(--semi-color-success)',
    })
    options.push({
      choice: 'theirsFull',
      name: t('conflict.choice.theirsFull'),
      flag: 'theirs-full',
      desc: t('conflict.choice.theirsFullDesc'),
      resultValue: displayValue(description.propertyValueIncomingNew, t),
      resultColor: 'var(--semi-color-primary)',
    })
    options.push({
      choice: 'base',
      name: t('conflict.choice.base'),
      flag: 'base',
      desc:
        description.operation === 'merge'
          ? t('conflict.choice.baseDescMerge')
          : t('conflict.choice.baseDescUpdate'),
      resultValue: displayValue(description.propertyValueIncomingOld, t),
      resultColor: 'var(--semi-color-purple)',
    })
    options.push({
      choice: 'merged',
      name: t('conflict.choice.merged'),
      flag: 'merged',
      desc: t('conflict.choice.mergedDesc'),
      resultValue: t('conflict.choice.resultCustom'),
      resultColor: 'var(--semi-color-warning)',
    })
  } else if (description.kind === 'text') {
    // Text conflict choices accepted by libsvn_wc
    // (build_text_conflict_resolve_items in conflicts.c):
    // - postpone: conflict remains (conflicts.c:1641)
    // - base: install the common-ancestor version (1648)
    // - mine_full / theirs_full: install my / the incoming full text
    //   (1658, 1653)
    // - mine_conflict / theirs_conflict: re-merge showing only my / only the
    //   incoming changes; require the merged-artifact file to exist,
    //   i.e. description.myAbsolutePath != null (1670-1698). Not available
    //   when a merged version cannot be created (e.g. binary conflicts)
    // - merged: install merged_file or the current local file (1707).
    //   Valid in SVN, but intentionally NOT offered yet — mergedFile
    //   handling for text conflicts is still undecided
    options.push({
      choice: 'postpone',
      name: t('conflict.choice.postpone'),
      flag: 'postpone',
      desc: t('conflict.choice.postponeDesc'),
      resultValue: t('conflict.choice.resultConflict'),
      resultColor: 'var(--semi-color-tertiary)',
    })
    options.push({
      choice: 'base',
      name: t('conflict.choice.baseText'),
      flag: 'base',
      desc: t('conflict.choice.baseTextDesc'),
      resultValue: t('conflict.choice.resultBaseFile'),
      resultColor: 'var(--semi-color-purple)',
    })
    options.push({
      choice: 'mineFull',
      name: t('conflict.choice.mineFull'),
      flag: 'mine-full',
      desc: t('conflict.choice.mineFullDescText'),
      resultValue: t('conflict.choice.resultMineWorking'),
      resultColor: 'var(--semi-color-success)',
    })
    options.push({
      choice: 'theirsFull',
      name: t('conflict.choice.theirsFull'),
      flag: 'theirs-full',
      desc: t('conflict.choice.theirsFullDescText'),
      resultValue: t('conflict.choice.resultTheirs'),
      resultColor: 'var(--semi-color-primary)',
    })
    if (description.myAbsolutePath) {
      options.push({
        choice: 'mineConflict',
        name: t('conflict.choice.mineConflict'),
        flag: 'mine-conflict',
        desc: t('conflict.choice.mineConflictDesc'),
        resultValue: t('conflict.choice.resultMineConflict'),
        resultColor: 'var(--semi-color-success)',
      })
      options.push({
        choice: 'theirsConflict',
        name: t('conflict.choice.theirsConflict'),
        flag: 'theirs-conflict',
        desc: t('conflict.choice.theirsConflictDesc'),
        resultValue: t('conflict.choice.resultTheirsConflict'),
        resultColor: 'var(--semi-color-primary)',
      })
    }
    // merged 选项暂时屏蔽：文本冲突的 merged 需要提供 mergedFile，
    // mergedFile 的处理方案尚未确定，先不暴露该选项
  } else if (description.kind === 'tree') {
    // Tree conflict choices accepted by libsvn_wc
    // (resolve_tree_conflict_on_node in conflicts.c):
    // - postpone: always valid, skips resolution entirely (conflicts.c:2075)
    // - merged: valid for ALL tree conflicts — the generic branch
    //   (conflicts.c:2884-2900) accepts merged and just clears the conflict
    //   marker; the moved-away branches accept it as well
    // - mine_conflict: only valid for update/switch with
    //   reason deleted/replaced (conflicts.c:2743) or
    //   reason moved_away + action edit (conflicts.c:2817)
    // - everything else (theirs_full, base, ...) is rejected by libsvn_wc

    options.push({
      choice: 'postpone',
      name: t('conflict.choice.postpone'),
      flag: 'postpone',
      desc: t('conflict.choice.postponeDesc'),
      resultValue: t('conflict.choice.resultConflict'),
      resultColor: 'var(--semi-color-tertiary)',
    })

    const isUpdateOrSwitch =
      description.operation === 'update' || description.operation === 'switch'
    const isMovedAwayEdit = description.reason === 'movedAway' && description.action === 'edit'

    if (
      isUpdateOrSwitch &&
      (description.reason === 'deleted' || description.reason === 'replaced' || isMovedAwayEdit)
    ) {
      options.push({
        choice: 'mineConflict',
        name: t('conflict.choice.keepLocalState'),
        flag: 'mine-conflict',
        desc: isMovedAwayEdit
          ? t('conflict.choice.keepLocalStateMovedDesc')
          : t('conflict.choice.keepLocalStateDesc'),
        resultValue: isMovedAwayEdit
          ? t('conflict.choice.resultMovedUpdated')
          : t('conflict.choice.resultLocalState'),
        resultColor: 'var(--semi-color-success)',
      })
    }

    const involvesMove =
      isMovedAwayEdit ||
      description.reason === 'deleted' ||
      description.reason === 'replaced' ||
      description.reason === 'movedAway'

    options.push({
      choice: 'merged',
      name: t('conflict.choice.acceptCurrent'),
      flag: 'merged',
      desc: involvesMove
        ? t('conflict.choice.acceptCurrentMovedDesc')
        : t('conflict.choice.acceptCurrentDesc'),
      resultValue: t('conflict.choice.resultCurrentState'),
      resultColor: 'var(--semi-color-warning)',
    })
  }

  return options
}

function ChoiceButtons({
  description,
  selectedChoice,
  onSelect,
}: {
  description: WorkingCopyConflictDescription
  selectedChoice: WorkingCopyConflictChoice
  onSelect: (c: WorkingCopyConflictChoice) => void
}) {
  const t = useT()
  const options = getChoiceOptions(description, t)

  return (
    <div className={cx(flex_col, gap_y_2)}>
      <h3 className={section_title}>{t('conflict.choice.title')}</h3>
      <div className={options_grid}>
        {options.map((opt) => (
          <button
            key={opt.choice}
            className={cx(opt_btn, selectedChoice === opt.choice && opt_btn_selected)}
            onClick={() => onSelect(opt.choice)}
          >
            <div className={opt_name}>
              {opt.name}
              <span className={opt_flag}>{opt.flag}</span>
            </div>
            <div className={opt_desc}>{opt.desc}</div>
            <div className={opt_result}>
              {t('conflict.choice.resultLabel')}{' '}
              <code
                style={{ color: opt.resultColor, background: 'rgba(var(--semi-grey-8), 0.06)' }}
              >
                {opt.resultValue}
              </code>
            </div>
          </button>
        ))}
      </div>
    </div>
  )
}

// ==================== Main Dialog ====================

export function NiceConflictDialog(props: {
  id: number
  description: WorkingCopyConflictDescription
}) {
  const modal = useCurrentModal()

  return (
    <ConflictDialog
      visible={modal.visible}
      afterClose={modal.remove}
      onOk={async () => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={async () => {
        modal.resolve(false)
        modal.hide()
      }}
      {...props}
    ></ConflictDialog>
  )
}

export default function ConflictDialog(props: ConflictDialogProps) {
  const t = useT()
  const [choice, setChoice] = useState<WorkingCopyConflictChoice>('postpone')
  const [propertyValue, setPropertyValue] = useState<string>('')
  const description = props.description

  const onOk = async () => {
    const value: WorkingCopyConflictResult = {
      choice,
      mergedFile: null,
      saveMerged: false,
      mergedValue: null,
    }
    if (description.kind === 'property' && choice === 'merged') {
      // merged requires a non-null merged_value on the SVN side
      // (conflicts.c:1389-1394 errors out without one); empty string is a
      // legitimate property value, so send the input as-is
      value.mergedValue = propertyValue
    }
    // const message: ReplyMessage = {
    //   success: value,
    // }
    await replySuccess(props.id, value)
    props.onOk?.()
  }

  const onCancel = async () => {
    const value: WorkingCopyConflictResult = {
      choice: 'postpone',
      mergedFile: null,
      saveMerged: false,
      mergedValue: null,
    }
    // const message: ReplyMessage = {
    //   success: value,
    // }
    await replySuccess(props.id, value)
    props.onCancel?.()
  }

  // Body content based on kind
  let bodyContent: React.ReactNode = null
  if (description.kind === 'text') {
    bodyContent = <TextBody description={description} />
  } else if (description.kind === 'property') {
    bodyContent = <PropertyBody description={description} />
  } else if (description.kind === 'tree') {
    bodyContent = <TreeBody description={description} />
  }

  return (
    <Dialog
      title={t('conflict.title')}
      onOk={onOk}
      onCancel={onCancel}
      afterClose={props.afterClose}
      visible={props.visible}
      size="large"
    >
      <ScrollArea className={cx(flex_1)} contentClassName={cx(flex, border_box, p_1)}>
        <div className={cx(flex_1, flex, flex_col, gap_y_3)}>
          {/* ① 头部（三种冲突通用） */}
          <Header description={description} />

          {/* ② 主体（按 kind 切换） */}
          {bodyContent}

          {/* ③ 解决按钮区 */}
          <ChoiceButtons description={description} selectedChoice={choice} onSelect={setChoice} />

          {/* 自定义值输入（仅属性冲突 + merged 选项时显示） */}
          {description.kind === 'property' && choice === 'merged' && (
            <DialogFormItem title={t('conflict.property.customValueLabel')}>
              <AutoComplete
                value={propertyValue}
                onChange={(e) => {
                  if (typeof e === 'string') {
                    setPropertyValue(e)
                  }
                }}
                data={[
                  description.propertyValueBase,
                  description.propertyValueWorking,
                  description.propertyValueIncomingOld,
                  description.propertyValueIncomingNew,
                ]
                  .filter((v): v is string => v !== null && v !== undefined)
                  .map((v) => ({ value: v, label: v }))}
                placeholder={t('conflict.property.customValuePlaceholder')}
                className={cx(flex_1)}
              />
            </DialogFormItem>
          )}
        </div>
      </ScrollArea>
    </Dialog>
  )
}
