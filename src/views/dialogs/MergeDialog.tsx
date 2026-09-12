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
import { useCurrentModal, useModal } from '@/lib/multi-modal'
import { box_shadow } from '@/styles/Components'
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
        content: 'Target must not be empty',
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
              content: 'Merge successful',
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
      title="Merge"
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
        <Typography.Text type="tertiary">
          将仓库中的变更应用到目标工作副本：先选择合并方式，再指定变更来源与合并目标。
        </Typography.Text>

        <div className={cx(flex, flex_col, gap_y_1)}>
          <span className={section_title}>合并方式</span>
          <RadioGroup
            value={sourceKind}
            onChange={(e) => setSourceKind(e.target.value as SourceKind)}
            className={cx(flex, flex_col, gap_y_1)}
          >
            <Radio value="range" className={mode_radio}>
              <span className={cx(flex, flex_col, gap_y_1)}>
                <span>Merge a range of revisions</span>
                <span className={hint}>
                  将来源分支在指定版本范围内的变更应用到目标，是最常用的合并方式。
                </span>
              </span>
            </Radio>
            <Radio value="trees" className={mode_radio}>
              <span className={cx(flex, flex_col, gap_y_1)}>
                <span>Merge two different trees</span>
                <span className={hint}>
                  比较两个来源的差异，并把差异应用到目标，常用于分支同步或反向合并。
                </span>
              </span>
            </Radio>
          </RadioGroup>
        </div>

        <div className={cx(flex, flex_col, gap_y_2)}>
          <span className={section_title}>合并来源</span>
          {sourceKind === 'range' ? (
            <div className={cx(flex, flex_col, gap_y_2)}>
              <DialogFormItem title="Source:">
                <PureInput
                  autoFocus
                  value={source}
                  onChange={setSource}
                  placeholder="仓库 URL 或本地路径"
                />
              </DialogFormItem>
              <div className={cx(flex, flex_col, box_shadow, p_2, border_box, border_radius_5)}>
                <DialogFormItem
                  title="Peg (可选，来源路径曾改名时才需要):"
                  wrapperClassName={cx(flex)}
                >
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
                          title="Start:"
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
                          title="End:"
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
                    Add range
                  </Button>
                </div>
              </div>
              <span className={hint}>起始版本之后、直到结束版本的变更将被合并到目标。</span>
            </div>
          ) : (
            <div className={cx(flex, flex_col, gap_y_2)}>
              <div className={cx(flex, gap_x_2)}>
                <div className={cx(flex_1, flex, flex_col, gap_y_1)}>
                  <DialogFormItem title="Source 1 (比较基准):">
                    <PureInput
                      autoFocus
                      value={source1}
                      onChange={setSource1}
                      placeholder="较旧树的 URL 或路径"
                    />
                  </DialogFormItem>
                  <DialogFormItem title="Revision 1:" wrapperClassName={cx(flex)}>
                    <RevisionSelect
                      className={cx(flex_1)}
                      kinds={['head', 'number', 'date']}
                      value={revision1}
                      onChange={setRevision1}
                    />
                  </DialogFormItem>
                </div>
                <div className={cx(flex_1, flex, flex_col, gap_y_1)}>
                  <DialogFormItem title="Source 2 (比较目标):">
                    <PureInput
                      value={source2}
                      onChange={setSource2}
                      placeholder="较新树的 URL 或路径"
                    />
                  </DialogFormItem>
                  <DialogFormItem title="Revision 2:" wrapperClassName={cx(flex)}>
                    <RevisionSelect
                      className={cx(flex_1)}
                      kinds={['head', 'number', 'date']}
                      value={revision2}
                      onChange={setRevision2}
                    />
                  </DialogFormItem>
                </div>
              </div>
              <span className={hint}>Source 1 到 Source 2 之间的差异将被应用到目标。</span>
            </div>
          )}
        </div>

        <div className={cx(flex, flex_col, gap_y_2)}>
          <span className={section_title}>合并目标</span>
          <DialogFormItem title="Target (接受合并的工作副本):">
            <PathInput value={target} onSelected={setTarget} onChange={setTarget} />
          </DialogFormItem>
          <DialogFormItem title="Depth:" wrapperClassName={cx(flex)}>
            <DepthSelect className={cx(flex_1)} onChange={setDepth} value={depth} />
          </DialogFormItem>
        </div>

        <Collapse>
          <Collapse.Panel header="高级选项" itemKey="advanced">
            <div className={cx(grid, grid_cols_1fr_1fr, gap_x_2, gap_y_2)}>
              <OptionCheckbox
                label="Ignore Merge Info"
                description="忽略已合并记录，允许重复合并相同变更"
                checked={ignoreMergeInfo}
                onChange={setIgnoreMergeInfo}
              />
              <OptionCheckbox
                label="Ignore Ancestry"
                description="忽略血缘关系，仅按内容差异比较"
                checked={ignoreAncestry}
                onChange={setIgnoreAncestry}
              />
              <OptionCheckbox
                label="Force Delete"
                description="合并时强制删除存在本地修改的文件"
                checked={forceDelete}
                onChange={setForceDelete}
              />
              <OptionCheckbox
                label="Record Only"
                description="仅记录合并信息，不修改工作副本内容"
                checked={recordOnly}
                onChange={setRecordOnly}
              />
              <OptionCheckbox
                label="Dry Run"
                description="试运行，只预览将产生的变更而不实际修改"
                checked={dryRun}
                onChange={setDryRun}
              />
              <OptionCheckbox
                label="Allow Mixed Revision"
                description="允许工作副本处于混合版本时操作"
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
