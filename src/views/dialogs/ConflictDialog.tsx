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

const badge_text = css`
  background: rgba(56, 189, 248, 0.15);
  color: #38bdf8;
`

const badge_property = css`
  background: rgba(251, 191, 36, 0.15);
  color: #fbbf24;
`

const badge_tree = css`
  background: rgba(192, 132, 252, 0.15);
  color: #c084fc;
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

function actionLabel(action: string): string {
  switch (action) {
    case 'edit':
      return '修改'
    case 'add':
      return '添加'
    case 'delete':
      return '删除'
    case 'replace':
      return '替换'
    default:
      return action
  }
}

function reasonLabel(reason: string): string {
  switch (reason) {
    case 'edited':
      return '已编辑'
    case 'obstructed':
      return '被阻挡'
    case 'deleted':
      return '已删除'
    case 'missing':
      return '缺失'
    case 'unversioned':
      return '未版本化'
    case 'added':
      return '已添加'
    case 'replaced':
      return '已替换'
    case 'movedAway':
      return '已移走'
    case 'movedHere':
      return '移入此处'
    default:
      return reason
  }
}

function operationLabel(op: string): string {
  switch (op) {
    case 'update':
      return 'update 时'
    case 'switch':
      return 'switch 时'
    case 'merge':
      return 'merge 时'
    case 'none':
      return ''
    default:
      return op
  }
}

function nodeKindLabel(nk: string): string {
  switch (nk) {
    case 'file':
      return '文件'
    case 'directory':
      return '目录'
    case 'symlink':
      return '符号链接'
    case 'none':
      return '不存在'
    default:
      return nk
  }
}

function versionSummary(v: WorkingCopyConflictVersion | null): string {
  if (!v) return '(不可用)'
  const rev = v.pegRevision
  const path = v.pathInRepository
  return `${path}@${rev} (${nodeKindLabel(v.nodeKind)})`
}

function displayValue(val: string | null): string {
  if (val === null || val === undefined) return '(未设置)'
  if (val === '') return '(空)'
  return `"${val}"`
}

// ==================== Sub-components ====================

function Header({ description }: { description: WorkingCopyConflictDescription }) {
  const op = operationLabel(description.operation)
  const summary = `对方${actionLabel(description.action)} × 本地${reasonLabel(description.reason)}`

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
  const files: { label: string; path: string | null; colorClass: string }[] = [
    { label: 'BASE · 本地基线', path: description.baseAbsolutePath, colorClass: val_base },
    { label: 'MINE · 我的修改', path: description.myAbsolutePath, colorClass: val_working },
    {
      label: 'THEIR · 对方版本',
      path: description.theirAbsolutePath,
      colorClass: val_incoming_new,
    },
    { label: 'MERGED · 合并结果', path: description.mergedFile, colorClass: val_incoming_old },
  ]

  return (
    <div className={cx(flex_col, gap_y_2)}>
      {description.isBinary && (
        <div className={binary_banner}>
          <span>⚠</span>
          <span>
            此文件为二进制文件 (MIME: <code className={mono}>{description.mimeType ?? '未知'}</code>
            )，文本 diff 不可用。
          </span>
        </div>
      )}
      <h3 className={section_title}>冲突文件</h3>
      <div className={cx(flex, flex_col, gap_x_2, gap_y_2)}>
        {files.map((f) => (
          <div key={f.label} className={cx(file_card, f.colorClass)}>
            <span className={file_card_label}>{f.label}</span>
            <span className={file_card_path}>{f.path ?? '(不可用)'}</span>
          </div>
        ))}
      </div>
    </div>
  )
}

function PropertyBody({ description }: { description: WorkingCopyConflictDescription }) {
  const propName = description.propertyName ?? '(unknown)'

  // Action summary
  const incomingOld = description.propertyValueIncomingOld
  const incomingNew = description.propertyValueIncomingNew
  const working = description.propertyValueWorking

  const incomingSummary =
    description.action === 'edit'
      ? `修改属性 "${propName}"：${displayValue(incomingOld)} → ${displayValue(incomingNew)}`
      : description.action === 'add'
        ? `添加属性 "${propName}" = ${displayValue(incomingNew)}`
        : description.action === 'delete'
          ? `删除属性 "${propName}"（原值 ${displayValue(incomingOld)}）`
          : `${description.action} 属性 "${propName}"`

  const localSummary =
    description.reason === 'edited'
      ? `已修改为 ${displayValue(working)}`
      : description.reason === 'added'
        ? `已添加（当前值 ${displayValue(working)}）`
        : `${reasonLabel(description.reason)}`

  const values = [
    {
      role: 'BASE · 本地基线',
      field: 'propertyValueBase',
      value: description.propertyValueBase,
      cls: val_base,
      note:
        description.operation === 'merge'
          ? '工作副本 BASE 版本（WC pristine）的值。我的血统起点。'
          : '在 update/switch 中，此值可能不是共同祖先；选择 base 时实际落地 incoming old。',
    },
    {
      role: 'WORKING · 我的修改',
      field: 'propertyValueWorking',
      value: description.propertyValueWorking,
      cls: val_working,
      note: '我在本地改成的值（未提交）。',
    },
    {
      role: 'INCOMING OLD · 对方起点',
      field: 'propertyValueIncomingOld',
      value: description.propertyValueIncomingOld,
      cls: val_incoming_old,
      note: '合并源 merge-left 的值。对方血统起点 / 补丁锚点。',
    },
    {
      role: 'INCOMING NEW · 对方新值',
      field: 'propertyValueIncomingNew',
      value: description.propertyValueIncomingNew,
      cls: val_incoming_new,
      note: '合并源 merge-right 的值。对方变更的终点。',
    },
  ]

  return (
    <div className={cx(flex_col, gap_y_2)}>
      <h3 className={section_title}>
        属性 <code className={mono}>"{propName}"</code>
      </h3>
      <div className={cx(flex_col, gap_y_1, p_1)}>
        <div className={text_12px}>
          <span style={{ color: 'rgba(var(--semi-blue-5), 1)', fontWeight: 600 }}>对方: </span>
          {incomingSummary}
          <span className={opt_flag} style={{ marginLeft: 6 }}>
            (action={description.action})
          </span>
        </div>
        <div className={text_12px}>
          <span style={{ color: 'rgba(var(--semi-green-5), 1)', fontWeight: 600 }}>我方: </span>
          {localSummary}
          <span className={opt_flag} style={{ marginLeft: 6 }}>
            (reason={description.reason})
          </span>
        </div>
      </div>

      <h3 className={section_title}>冲突中的四个值</h3>
      <div className={cx(grid_cols_1fr_1fr, gap_x_2, gap_y_2)}>
        {values.map((v) => (
          <div key={v.field} className={cx(val_card, v.cls)}>
            <div className={val_card_role}>
              <span>{v.role}</span>
              <span className={val_card_field}>{v.field}</span>
            </div>
            <code className={val_card_code}>{displayValue(v.value)}</code>
            <div className={val_card_note}>{v.note}</div>
          </div>
        ))}
      </div>

      <div className={trap_box}>
        <b>⚠ 语义陷阱：</b>在 merge 场景下，<code className={mono}>propertyValueBase</code> 显示的是
        本地 pristine 的值，但若选择 <b>base</b>，实际落地的是{' '}
        <code className={mono}>propertyValueIncomingOld</code>（对方的起点值）。两者不一致。
      </div>
    </div>
  )
}

function TreeBody({ description }: { description: WorkingCopyConflictDescription }) {
  const localText = `本地 ${nodeKindLabel(description.nodeKind)} ${reasonLabel(description.reason)}`
  const incomingNodeKind =
    description.action === 'edit' || description.action === 'delete'
      ? (description.sourceLeftVersion?.nodeKind ?? 'unknown')
      : (description.sourceRightVersion?.nodeKind ?? 'unknown')
  const incomingText = `传入 ${nodeKindLabel(incomingNodeKind)} ${actionLabel(description.action)}`

  return (
    <div className={cx(flex_col, gap_y_2)}>
      <h3 className={section_title}>冲突概述</h3>
      <div className={tree_summary}>
        <div className={tree_row}>
          <span className={tree_row_label}>本地:</span>
          <span>{localText}</span>
        </div>
        <div className={tree_row}>
          <span className={tree_row_label}>传入:</span>
          <span>{incomingText}</span>
        </div>
        <div className={tree_row}>
          <span className={tree_row_label}>操作:</span>
          <span>
            {description.operation !== 'none' ? `upon ${description.operation}` : '无特定操作'}
          </span>
        </div>
      </div>

      <h3 className={section_title}>版本信息</h3>
      <div className={cx(flex, flex_col, gap_x_2, gap_y_1)}>
        <div className={version_card}>
          <span className={version_card_label}>merge-left (src_left_version)</span>
          <span className={version_card_detail}>
            {description.sourceLeftVersion
              ? versionSummary(description.sourceLeftVersion)
              : '(不可用 — 老式冲突记录)'}
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
              ? versionSummary(description.sourceRightVersion)
              : '(不可用 — 老式冲突记录)'}
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

function getChoiceOptions(description: WorkingCopyConflictDescription): ChoiceOption[] {
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
      name: '推迟解决',
      flag: 'postpone',
      desc: '暂不处理，稍后手动解决。',
      resultValue: '保持冲突状态',
      resultColor: 'var(--semi-color-tertiary)',
    })
    options.push({
      choice: 'mineFull',
      name: '保留我的',
      flag: 'mine-full',
      desc: '保留我的本地修改，拒绝传入变更。',
      resultValue: displayValue(description.propertyValueWorking),
      resultColor: 'var(--semi-color-success)',
    })
    options.push({
      choice: 'theirsFull',
      name: '采用对方',
      flag: 'theirs-full',
      desc: '整体采用对方新值，放弃我的修改。',
      resultValue: displayValue(description.propertyValueIncomingNew),
      resultColor: 'var(--semi-color-primary)',
    })
    options.push({
      choice: 'base',
      name: '回到变更前',
      flag: 'base',
      desc:
        description.operation === 'merge'
          ? '回到变更发生前的状态（实际落地 BASE 版本的属性值）。'
          : '回到变更发生前的状态（实际落地 incoming old 值）。',
      resultValue: displayValue(description.propertyValueIncomingOld),
      resultColor: 'var(--semi-color-purple)',
    })
    options.push({
      choice: 'merged',
      name: '自定义值…',
      flag: 'merged',
      desc: '输入任意值（通过 merged_value 返回）。',
      resultValue: '自定义',
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
      name: '推迟解决',
      flag: 'postpone',
      desc: '暂不处理，稍后手动解决。',
      resultValue: '保持冲突状态',
      resultColor: 'var(--semi-color-tertiary)',
    })
    options.push({
      choice: 'base',
      name: '回到基线',
      flag: 'base',
      desc: '使用 BASE 版本（共同祖先）的文件。',
      resultValue: 'BASE 文件',
      resultColor: 'var(--semi-color-purple)',
    })
    options.push({
      choice: 'mineFull',
      name: '保留我的',
      flag: 'mine-full',
      desc: '保留我的工作副本文件，拒绝传入变更。',
      resultValue: '我的工作副本',
      resultColor: 'var(--semi-color-success)',
    })
    options.push({
      choice: 'theirsFull',
      name: '采用对方',
      flag: 'theirs-full',
      desc: '整体采用对方的文件版本。',
      resultValue: '对方版本',
      resultColor: 'var(--semi-color-primary)',
    })
    if (description.myAbsolutePath) {
      options.push({
        choice: 'mineConflict',
        name: '只应用我的改动',
        flag: 'mine-conflict',
        desc: '重新执行三方合并，仅保留我这边的改动，丢弃对方的改动。',
        resultValue: '合并结果（仅我的改动）',
        resultColor: 'var(--semi-color-success)',
      })
      options.push({
        choice: 'theirsConflict',
        name: '只应用对方的改动',
        flag: 'theirs-conflict',
        desc: '重新执行三方合并，仅保留对方那边的改动，丢弃我的改动。',
        resultValue: '合并结果（仅对方改动）',
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
      name: '推迟解决',
      flag: 'postpone',
      desc: '暂不处理，稍后手动解决。',
      resultValue: '保持冲突状态',
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
        name: '保留本地状态',
        flag: 'mine-conflict',
        desc: isMovedAwayEdit
          ? '更新被移走的节点以应用传入变更。'
          : '保留本地目录状态，并对从其中移出的子节点产生新的 moved-away 冲突。',
        resultValue: isMovedAwayEdit ? '更新后的移走节点' : '本地状态',
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
      name: '接受当前状态',
      flag: 'merged',
      desc: involvesMove
        ? '接受当前工作副本状态（如有需要会断开移动记录）。'
        : '接受当前工作副本状态，仅清除冲突标记。',
      resultValue: '当前状态',
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
  const options = getChoiceOptions(description)

  return (
    <div className={cx(flex_col, gap_y_2)}>
      <h3 className={section_title}>选择一个解决方式</h3>
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
              结果 →{' '}
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
      title="解决冲突"
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
            <DialogFormItem title="自定义属性值:">
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
                placeholder="输入自定义值…"
                className={cx(flex_1)}
              />
            </DialogFormItem>
          )}
        </div>
      </ScrollArea>
    </Dialog>
  )
}
