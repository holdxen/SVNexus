import { IconMinus, IconPlus } from '@douyinfe/semi-icons'
import {
  Button,
  Checkbox,
  Collapse,
  Divider,
  Radio,
  RadioGroup,
  Toast,
  Typography,
} from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { useRef, useState } from 'react'

import { Depth } from '@/bindings/Depth'
import { MergeOptions } from '@/bindings/MergeOptions'
import { MergeSource } from '@/bindings/MergeSource'
import { Revision } from '@/bindings/Revision'
import { RevisionRange } from '@/bindings/RevisionRange'
import PathInput from '@/components/PathInput'
import PureInput from '@/components/PureInput'
import { ScrollArea } from '@/components/ScrollArea'
import DepthSelect from '@/components/subversion/DepthSelect'
import RevisionSelect from '@/components/subversion/RevisionSelect'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal, useModal } from '@/lib/multi-modal'
import {
  border_box,
  border_radius_5,
  flex,
  flex_1,
  flex_col,
  flex_row_reverse,
  gap_x_2,
  gap_y_1,
  gap_y_2,
  gap_y_3,
  grid,
  grid_cols_1fr_1fr,
  hidden,
  my_1,
  p_2,
  self_center,
} from '@/styles/Classes'
import { box_shadow } from '@/styles/Components'

import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'
import { LoadingDialogRef, NiceLoadingDialog } from './LoadingDialog'

export function NiceMergeDialog(props: { defaultTarget?: string; defaultSource?: MergeSource }) {
  const modal = useCurrentModal()
  return (
    <MergeDialog
      defaultTarget={props.defaultTarget}
      defaultSource={props.defaultSource}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
    ></MergeDialog>
  )
}

export interface MergeDialogProps {
  visible: boolean
  afterClose?: () => void
  onOk?: () => void
  onCancel?: () => void
  defaultTarget?: string
  defaultSource?: MergeSource
}

type SourceKind = 'range' | 'trees'

const section_title = css`
  font-size: 13px;
  font-weight: 600;
  color: rgba(var(--semi-grey-9), 1);
`

const hint = css`
  font-size: 12px;
  line-height: 1.5;
  color: rgba(var(--semi-grey-7), 1);
`

const mode_radio = css`
  align-items: flex-start;
  padding: 8px 10px;
  margin-right: 0;
  border-radius: 6px;
  &:hover {
    background-color: var(--semi-color-fill-0);
  }
`

function OptionCheckbox(props: {
  label: string
  description: string
  checked: boolean
  onChange: (checked: boolean) => void
}) {
  return (
    <Checkbox checked={props.checked} onChange={(e) => props.onChange(e.target.checked ?? false)}>
      <span className={cx(flex, flex_col, gap_y_1)}>
        <span>{props.label}</span>
        <span className={hint}>{props.description}</span>
      </span>
    </Checkbox>
  )
}

// function mergeSource1(source: MergeSource): string | undefined {
//   if ('target' in source) {
//     return source.target.source1
//   }
//   return undefined
// }

function nullOr<T, R>(value: T | undefined, convert: (value: T) => R): R | undefined {
  if (value === undefined) return undefined
  return convert(value)
}

function mergeSourceTarget<T>(
  source: MergeSource,
  convert: (target: Extract<MergeSource, { target: any }>['target']) => T,
): T | undefined {
  if ('target' in source) {
    return convert(source.target)
  }
  return undefined
}

function mergeSourcePeg<T>(
  source: MergeSource,
  convert: (peg: Extract<MergeSource, { peg: any }>['peg']) => T,
): T | undefined {
  if ('peg' in source) {
    return convert(source.peg)
  }
  return undefined
}

export default function MergeDialog(props: MergeDialogProps) {
  const t = useT()
  const [target, setTarget] = useState(props.defaultTarget ?? '')
  const [depth, setDepth] = useState<Depth>('infinity')
  const [ignoreMergeInfo, setIgnoreMergeInfo] = useState(false)
  const [ignoreAncestry, setIgnoreAncestry] = useState(false)
  const [forceDelete, setForceDelete] = useState(false)
  const [recordOnly, setRecordOnly] = useState(false)
  const [dryRun, setDryRun] = useState(false)
  const [allowMixedRevision, setAllowMixedRevision] = useState(false)

  const defaultSource = props.defaultSource

  const [source, setSource] = useState(
    nullOr(defaultSource, (value) => {
      return mergeSourcePeg(value, (peg) => peg.source)
    }) ?? '',
  )
  // const [startRevision, setStartRevision] = useState<Revision>(
  //   nullOr(defaultSource, (value) => {
  //     return mergeSourcePeg(value, (peg) =>
  //       peg.rangesToMerge?.length === 1 ? peg.rangesToMerge[0].start : undefined,
  //     )
  //   }) ?? 'head',
  // )
  // console.log(
  //   'end revision is',
  //   nullOr(defaultSource, (value) => {
  //     return mergeSourcePeg(value, (peg) =>
  //       peg.rangesToMerge?.length === 1 ? peg.rangesToMerge[0].end : undefined,
  //     )
  //   }) ?? 'head',
  // )
  // const [endRevision, setEndRevision] = useState<Revision>(
  //   nullOr(defaultSource, (value) => {
  //     return mergeSourcePeg(value, (peg) =>
  //       peg.rangesToMerge?.length === 1 ? peg.rangesToMerge[0].end : undefined,
  //     )
  //   }) ?? 'head',
  // )
  const [pegRevision, setPegRevision] = useState<Revision>(
    nullOr(defaultSource, (value) => {
      return mergeSourcePeg(value, (peg) => peg.pegRevision)
    }) ?? 'head',
  )

  const [source1, setSource1] = useState(
    nullOr(defaultSource, (value) => {
      return mergeSourceTarget(value, (target) => target.source1)
    }) ?? '',
  )
  const [source2, setSource2] = useState(
    nullOr(defaultSource, (value) => {
      return mergeSourceTarget(value, (target) => target.source2)
    }) ?? '',
  )
  const [revision1, setRevision1] = useState<Revision>(
    nullOr(defaultSource, (value) => {
      return mergeSourceTarget(value, (target) => target.revision1)
    }) ?? 'head',
  )
  const [revision2, setRevision2] = useState<Revision>(
    nullOr(defaultSource, (value) => {
      return mergeSourceTarget(value, (target) => target.revision2)
    }) ?? 'head',
  )

  const [sourceKind, setSourceKind] = useState<SourceKind>(
    defaultSource === undefined || 'peg' in defaultSource ? 'range' : 'trees',
  )

  const defaultRevisionRange: RevisionRange = { start: 'head', end: 'head' }

  const [revisionRanges, setRevisionRanges] = useState<RevisionRange[]>(
    nullOr(defaultSource, (value) => {
      return mergeSourcePeg(value, (target) => target.rangesToMerge ?? [])
    }) ?? [defaultRevisionRange],
  )

  const modal = useModal()

  const loadingDialog = useRef<LoadingDialogRef>(null)

  const subversion = useSubversion()

  const onOk = () => {
    if (target === '') {
      Toast.error({
        content: t('shared.error.targetRequired'),
        stack: false,
      })
      return
    }

    const mergeSource: MergeSource =
      sourceKind === 'range'
        ? {
            peg: {
              source,
              pegRevision,
              rangesToMerge: revisionRanges.length === 0 ? null : revisionRanges,
            },
          }
        : {
            target: {
              source1,
              revision1,
              source2,
              revision2,
            },
          }

    const options: MergeOptions = {
      source: mergeSource,
      target,
      depth,
      ignoreMergeInfo,
      ignoreAncestry,
      forceDelete,
      recordOnly,
      dryRun,
      allowMixedRevision,
      extraMergeOptions: null,
    }

    modal.show(NiceLoadingDialog, {
      onLoad: () => {
        Subversion.callOnce({
          factory: subversion,
          async call(context) {
            await context.merge(options)
            Toast.success({
              content: t('shared.success.merge'),
              stack: true,
            })
            loadingDialog.current?.close()
            props.onOk?.()
          },
          onError(_, handler) {
            handler()
            loadingDialog.current?.close()
          },
        })
      },
      cancelable: true,
      ref: loadingDialog,
    })
  }

  return (
    <Dialog
      title={t('shared.action.merge')}
      size="medium"
      afterClose={props.afterClose}
      onOk={onOk}
      onCancel={props.onCancel}
      visible={props.visible}
    >
      <ScrollArea
        className={cx(flex_1)}
        contentClassName={cx(
          flex,
          flex_col,
          gap_y_3,
          border_box,
          css`
            margin-right: 3px;
            margin-left: 3px;
          `,
        )}
      >
        <Typography.Text type="tertiary">{t('merge.description')}</Typography.Text>

        <div className={cx(flex, flex_col, gap_y_1)}>
          <span className={section_title}>{t('merge.mode.title')}</span>
          <RadioGroup
            value={sourceKind}
            onChange={(e) => setSourceKind(e.target.value as SourceKind)}
            className={cx(flex, flex_col, gap_y_1)}
          >
            <Radio value="range" className={mode_radio}>
              <span className={cx(flex, flex_col, gap_y_1)}>
                <span>{t('advancedDialogs.merge.modeRange')}</span>
                <span className={hint}>{t('merge.mode.rangeHint')}</span>
              </span>
            </Radio>
            <Radio value="trees" className={mode_radio}>
              <span className={cx(flex, flex_col, gap_y_1)}>
                <span>{t('advancedDialogs.merge.modeTrees')}</span>
                <span className={hint}>{t('merge.mode.treesHint')}</span>
              </span>
            </Radio>
          </RadioGroup>
        </div>

        <div className={cx(flex, flex_col, gap_y_2)}>
          <span className={section_title}>{t('merge.source.title')}</span>
          {sourceKind === 'range' ? (
            <div className={cx(flex, flex_col, gap_y_2)}>
              <DialogFormItem title={t('shared.field.source')}>
                <PureInput
                  autoFocus
                  value={source}
                  onChange={setSource}
                  placeholder={t('merge.source.placeholder')}
                />
              </DialogFormItem>
              <div className={cx(flex, flex_col, box_shadow, p_2, border_box, border_radius_5)}>
                <DialogFormItem title={t('merge.source.pegTitle')} wrapperClassName={cx(flex)}>
                  <RevisionSelect
                    className={cx(flex_1)}
                    disableLayout
                    kinds={['head', 'number', 'date', 'base', 'working']}
                    value={pegRevision}
                    onChange={setPegRevision}
                  />
                </DialogFormItem>
                <Divider className={cx(my_1, border_box)}></Divider>
                <ScrollArea contentClassName={cx(flex, flex_col)}>
                  {revisionRanges.map((e, index) => {
                    return (
                      <div key={index} className={cx(flex, gap_x_2)}>
                        <DialogFormItem
                          title={t('shared.field.start')}
                          className={cx(flex_1)}
                          wrapperClassName={cx(flex)}
                        >
                          <RevisionSelect
                            className={cx(flex_1)}
                            kinds={['head', 'number', 'date']}
                            value={e.start}
                            onChange={(value) => {
                              setRevisionRanges((v) =>
                                v.map((e, i) => (i === index ? { ...e, start: value } : e)),
                              )
                            }}
                          />
                        </DialogFormItem>
                        <DialogFormItem
                          title={t('shared.field.end')}
                          className={cx(flex_1)}
                          wrapperClassName={cx(flex)}
                        >
                          <RevisionSelect
                            className={cx(flex_1)}
                            kinds={['head', 'number', 'date']}
                            value={e.end}
                            onChange={(value) => {
                              setRevisionRanges((v) =>
                                v.map((e, i) => (i === index ? { ...e, end: value } : e)),
                              )
                            }}
                          />
                        </DialogFormItem>
                        <Button
                          className={cx(self_center)}
                          onClick={() => {
                            setRevisionRanges((v) => v.filter((_, i) => i !== index))
                          }}
                          icon={<IconMinus />}
                        ></Button>
                      </div>
                    )
                  })}
                </ScrollArea>
                <Divider
                  className={cx(my_1, border_box, revisionRanges.length === 0 && hidden)}
                ></Divider>
                <div className={cx(flex, flex_row_reverse)}>
                  <Button
                    onClick={() => {
                      setRevisionRanges((values) => [...values, defaultRevisionRange])
                    }}
                    icon={<IconPlus />}
                  >
                    {t('shared.option.addRange')}
                  </Button>
                </div>
              </div>
              <span className={hint}>{t('merge.source.rangeHint')}</span>
            </div>
          ) : (
            <div className={cx(flex, flex_col, gap_y_2)}>
              <div className={cx(flex, gap_x_2)}>
                <div className={cx(flex_1, flex, flex_col, gap_y_1)}>
                  <DialogFormItem title={t('merge.source.source1Title')}>
                    <PureInput
                      autoFocus
                      value={source1}
                      onChange={setSource1}
                      placeholder={t('merge.source.olderTreePlaceholder')}
                    />
                  </DialogFormItem>
                  <DialogFormItem title={t('shared.field.revision1')} wrapperClassName={cx(flex)}>
                    <RevisionSelect
                      className={cx(flex_1)}
                      kinds={['head', 'number', 'date']}
                      value={revision1}
                      onChange={setRevision1}
                    />
                  </DialogFormItem>
                </div>
                <div className={cx(flex_1, flex, flex_col, gap_y_1)}>
                  <DialogFormItem title={t('merge.source.source2Title')}>
                    <PureInput
                      value={source2}
                      onChange={setSource2}
                      placeholder={t('merge.source.newerTreePlaceholder')}
                    />
                  </DialogFormItem>
                  <DialogFormItem title={t('shared.field.revision2')} wrapperClassName={cx(flex)}>
                    <RevisionSelect
                      className={cx(flex_1)}
                      kinds={['head', 'number', 'date']}
                      value={revision2}
                      onChange={setRevision2}
                    />
                  </DialogFormItem>
                </div>
              </div>
              <span className={hint}>{t('merge.source.treesHint')}</span>
            </div>
          )}
        </div>

        <div className={cx(flex, flex_col, gap_y_2)}>
          <span className={section_title}>{t('merge.target.title')}</span>
          <DialogFormItem title={t('merge.target.pathTitle')}>
            <PathInput value={target} onSelected={setTarget} onChange={setTarget} />
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.depth')} wrapperClassName={cx(flex)}>
            <DepthSelect className={cx(flex_1)} onChange={setDepth} value={depth} />
          </DialogFormItem>
        </div>

        <Collapse>
          <Collapse.Panel header={t('merge.advanced.title')} itemKey="advanced">
            <div className={cx(grid, grid_cols_1fr_1fr, gap_x_2, gap_y_2)}>
              <OptionCheckbox
                label={t('shared.option.ignoreMergeInfo')}
                description={t('merge.advanced.ignoreMergeInfoHint')}
                checked={ignoreMergeInfo}
                onChange={setIgnoreMergeInfo}
              />
              <OptionCheckbox
                label={t('shared.option.ignoreAncestry')}
                description={t('merge.advanced.ignoreAncestryHint')}
                checked={ignoreAncestry}
                onChange={setIgnoreAncestry}
              />
              <OptionCheckbox
                label={t('shared.option.forceDelete')}
                description={t('merge.advanced.forceDeleteHint')}
                checked={forceDelete}
                onChange={setForceDelete}
              />
              <OptionCheckbox
                label={t('shared.option.recordOnly')}
                description={t('merge.advanced.recordOnlyHint')}
                checked={recordOnly}
                onChange={setRecordOnly}
              />
              <OptionCheckbox
                label={t('shared.option.dryRun')}
                description={t('merge.advanced.dryRunHint')}
                checked={dryRun}
                onChange={setDryRun}
              />
              <OptionCheckbox
                label={t('shared.option.allowMixedRevision')}
                description={t('merge.advanced.allowMixedRevisionHint')}
                checked={allowMixedRevision}
                onChange={setAllowMixedRevision}
              />
            </div>
          </Collapse.Panel>
        </Collapse>
      </ScrollArea>
    </Dialog>
  )
}
