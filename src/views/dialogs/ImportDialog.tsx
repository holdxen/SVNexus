import { IconFolderStroked } from '@douyinfe/semi-icons'
import { Card, Checkbox, Descriptions, Progress, Select, Toast } from '@douyinfe/semi-ui'
import { Data } from '@douyinfe/semi-ui/lib/es/descriptions'
import { css, cx } from '@linaria/core'
import { open } from '@tauri-apps/plugin-dialog'
import { useMemoizedFn } from 'ahooks'
import dayjs from 'dayjs'
import { filesize } from 'filesize'
import { useCallback, useEffect, useRef, useState } from 'react'

import { Depth } from '@/bindings/Depth'
import { ImportFilterEvent } from '@/bindings/ImportFilterEvent'
import { ImportOptions } from '@/bindings/ImportOptions'
import PureInput from '@/components/PureInput'
import PureTextArea from '@/components/PureTextArea'
import { ScrollArea } from '@/components/ScrollArea'
import { replySuccess } from '@/context/Functions'
import { Subversion, SubversionEventMap, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import { useCurrentModal, useModal } from '@/lib/multi-modal'
import {
  cursor_pointer,
  flex,
  flex_1,
  flex_col,
  gap_y_1,
  gap_y_2,
  gap_y_3,
  hidden,
  min_h_0,
} from '@/styles/Classes'
import { disable_move } from '@/styles/Components'
import errorHumanString from '@/utils/Error'
import { MessagePackChannel } from '@/utils/MessagePack'
import { localPath } from '@/utils/Path'

import DepthSelect from '../../components/subversion/DepthSelect'
import ConfirmDialog from './ConfirmDialog'
import { Dialog } from './Dialog'
import DialogFormItem from './DialogFormItem'

const button = css`
  color: var(--semi-color-primary);
  &:hover {
    color: var(--semi-color-primary-hover);
  }
  &:active {
    color: var(--semi-color-primary-active);
  }
`

// Import 进度对话框：执行期间显示进度，支持取消
function ImportingDialog({
  options,
  filterType,
  filters,
}: {
  options: ImportOptions
  filterType: FilterType
  filters: string
}) {
  const t = useT()
  const subversion = useSubversion()
  const modal = useCurrentModal()

  const [revision, setRevision] = useState<number | null>(null)
  const [current, setCurrent] = useState('')
  const [pos, setPos] = useState<number>(-1)
  const [total, setTotal] = useState<number>(-1)
  const [finished, setFinished] = useState(false)
  const [cancel, setCancel] = useState<((msg: string) => Promise<any>) | null>(null)
  const [hasError, setHasError] = useState(false)

  const onProgressNotify = useCallback((data: SubversionEventMap['progressNotify']) => {
    setPos(data.pos)
    setTotal(data.total)
  }, [])
  const onWorkingNotify = useCallback((data: SubversionEventMap['workingCopyNotify']) => {
    setCurrent(localPath.getFileName(data.notify.path) ?? '')
  }, [])

  const cleanup = useMemoizedFn(() => {
    if (cancel !== null) {
      cancel('Destroyed').catch(() => {})
    }
  })

  const dialogModal = useModal()

  useEffect(() => {
    if (finished) {
      return
    }
    async function call() {
      await Subversion.callOnce({
        factory: subversion,
        call: async (context: Subversion) => {
          context.on('progressNotify', onProgressNotify)
          context.on('workingCopyNotify', onWorkingNotify)
          if (filterType === 'oneByOne') {
            const channel = new MessagePackChannel<ImportFilterEvent>()

            channel.onmessage = (event) => {
              const data: Data[] = [
                {
                  key: t('shared.field.path'),
                  value: event.path,
                },
                {
                  key: t('advancedDialogs.import.kind'),
                  value: event.kind,
                },
                {
                  key: t('advancedDialogs.import.size'),
                  value: event.fileSize === null ? '' : filesize(event.fileSize),
                  hidden: event.fileSize === null,
                },
                {
                  key: t('advancedDialogs.import.modifyTime'),
                  value: dayjs(event.mtime).format(),
                },
              ]
              const exec = async () => {
                const result = await dialogModal
                  .show(ConfirmDialog, {
                    title: t('advancedDialogs.import.confirmTitle'),
                    children: (
                      <div>
                        <Descriptions data={data}></Descriptions>
                      </div>
                    ),
                  })
                  .as<boolean>()
                await replySuccess(event.id, result)
              }
              exec()
            }

            const promise = context.importFilter(options, channel)
            setCancel(() => async (msg: string) => {
              await context?.cancel(msg)
              await promise
            })
            const result = await promise
            setRevision(result.info.revision)
          } else {
            const promise = context.import(options, filters.split(/\r?\n/))
            setCancel(() => async (msg: string) => {
              await context?.cancel(msg)
              await promise
            })
            const result = await promise
            setRevision(result.info.revision)
          }
        },
        onError: (error) => {
          Toast.error({
            content: t('advancedDialogs.import.failed', { error: errorHumanString(error) }),
            stack: true,
          })
          setHasError(true)
        },
        onFinally: (context) => {
          context?.off('progressNotify', onProgressNotify)
          context?.off('workingCopyNotify', onWorkingNotify)
        },
      })
      setFinished(true)
      setCancel(null)
    }

    call()
    return cleanup
  }, [])

  const descriptions = [
    { key: t('advancedDialogs.progress.path'), value: options.path },
    { key: t('advancedDialogs.progress.url'), value: options.url },
    { key: t('shared.column.revision'), value: revision?.toString() ?? null },
  ]

  const onCancel = async () => {
    if (cancel !== null) {
      try {
        await cancel('Cancelled by user')
      } catch (error) {}
    }
    setCancel(null)
  }

  const onOk = () => {
    modal.resolve(!hasError)
    modal.hide()
  }

  const percent = finished ? 100 : Math.max(Number((pos / total) * 100), 0)

  const isIndeterminate = finished ? false : pos < 0 || total < 0

  return (
    <Dialog
      afterClose={modal.remove}
      closeOnEsc={false}
      okButtonProps={finished ? undefined : { style: { display: 'none' } }}
      cancelButtonProps={finished ? { style: { display: 'none' } } : undefined}
      maskClosable={false}
      closable={false}
      size="medium"
      title={t('advancedDialogs.import.progressTitle')}
      visible={modal.visible}
      onCancel={onCancel}
      onOk={onOk}
    >
      <div className={cx(flex, flex_col, gap_y_3, disable_move)}>
        <Card>
          <Descriptions size="medium" data={descriptions}></Descriptions>
        </Card>
        <DialogFormItem title={t('shared.field.progress')}>
          <div className={cx(flex_1, flex_col)}>
            <div>{t('advancedDialogs.progress.current', { name: current })}</div>
            <div className={cx(flex)}>
              <div>{t('advancedDialogs.progress.percentage', { percent })}</div>
              <div className={cx(flex_1)}></div>
              <div>{`${pos < 0 ? t('advancedDialogs.progress.unknown') : String(pos)}/${total < 0 ? t('advancedDialogs.progress.unknown') : String(total)}`}</div>
            </div>
            <Progress indeterminate={isIndeterminate} percent={percent} size="large" />
          </div>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}

const inputIcon = css`
  .semi-input-append {
    cursor: default;
  }
`

export function NiceImportDialog(props: { defaultPath?: string }) {
  const modal = useCurrentModal()
  return (
    <ImportDialog
      defaultPath={props.defaultPath}
      visible={modal.visible}
      onOk={() => {
        modal.resolve(true)
        modal.hide()
      }}
      afterClose={modal.remove}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
    ></ImportDialog>
  )
}

export interface ImportDialogProps {
  visible: boolean
  onOk?: () => void
  onCancel?: () => void
  afterClose?: () => void
  defaultPath?: string
}

type FilterType = 'none' | 'oneByOne' | 'lines'

export default function ImportDialog(props: ImportDialogProps) {
  const t = useT()
  const [path, setPath] = useState(props.defaultPath ?? '')
  const [url, setUrl] = useState('')
  const [depth, setDepth] = useState<Depth>('infinity')
  const [noIgnore, setNoIgnore] = useState(false)
  const [noAutoprops, setNoAutoprops] = useState(false)
  const [ignoreUnknownNodeTypes, setIgnoreUnknownNodeTypes] = useState(false)
  const [commitMessage, setCommitMessage] = useState('')
  const [filterType, setFilterType] = useState<FilterType>('none')
  const [filters, setFilters] = useState('')

  const selectPath = async () => {
    const selected = await open({
      title: t('shared.action.selectFolderToImport'),
      multiple: false,
      directory: true,
    })
    if (selected === null) {
      return
    }
    setPath(selected.replace(/\\/g, '/'))
  }

  const onCancel = () => {
    props.onCancel?.()
  }

  const modal = useModal()

  const onOk = async () => {
    if (path === '') {
      Toast.error({
        content: t('shared.error.pathRequired'),
        stack: true,
      })
      return
    }
    if (url === '') {
      Toast.error({
        content: t('shared.error.urlRequired'),
        stack: true,
      })
      return
    }
    if (commitMessage === '') {
      Toast.error({
        content: t('shared.error.commitMessageEmpty'),
        stack: true,
      })
      return
    }

    const options: ImportOptions = {
      path,
      url,
      depth,
      noIgnore,
      noAutoprops,
      ignoreUnknownNodeTypes,
      revisionPropertyTable: null,
      commitMessage,
    }

    const result = await modal.show(ImportingDialog, { options, filterType, filters }).as<boolean>()

    if (result) {
      props.onOk?.()
    }
  }

  const focusElement = useRef(null)

  // const filter = <Select value={filterType}>
  //   <Select.Option value={'none'}>none</Select.Option>
  //   <Select.Option value={'oneByOne'}>oneByOne</Select.Option>
  //   <Select.Option value={'lines'}>lines</Select.Option>
  // </Select>

  return (
    <Dialog
      afterClose={props.afterClose}
      size="medium"
      title={t('shared.action.import')}
      visible={props.visible}
      onCancel={onCancel}
      onOk={onOk}
      initialFocusRef={focusElement}
    >
      <ScrollArea className={cx(flex_1, min_h_0)} contentClassName={cx(flex)}>
        <div className={cx(flex_1, flex, gap_y_3, flex_col)}>
          <DialogFormItem title={t('shared.field.pathLocalFolder')}>
            <PureInput
              ref={focusElement}
              className={inputIcon}
              autoFocus
              value={path}
              onChange={(value) => setPath(value)}
              addonAfter={
                <IconFolderStroked onClick={selectPath} className={cx(cursor_pointer, button)} />
              }
            ></PureInput>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.url')}>
            <PureInput
              className={inputIcon}
              value={url}
              onChange={(value) => setUrl(value)}
              placeholder={'https://example.com/repository/trunk'}
            ></PureInput>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.depth')}>
            <DepthSelect
              value={depth}
              onChange={(value) => setDepth(value)}
              className={flex_1}
            ></DepthSelect>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.commitMessage')}>
            <PureTextArea
              value={commitMessage}
              onChange={(e) => setCommitMessage(e)}
            ></PureTextArea>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.options')}>
            <div className={cx(flex, flex_col, gap_y_2)}>
              <Checkbox checked={noIgnore} onChange={(e) => setNoIgnore(e.target.checked ?? false)}>
                {t('shared.option.includeIgnored')}
              </Checkbox>
              <Checkbox
                checked={noAutoprops}
                onChange={(e) => setNoAutoprops(e.target.checked ?? false)}
              >
                {t('shared.option.noAutoProps')}
              </Checkbox>
              <Checkbox
                checked={ignoreUnknownNodeTypes}
                onChange={(e) => setIgnoreUnknownNodeTypes(e.target.checked ?? false)}
              >
                {t('shared.option.ignoreUnknownNodeTypes')}
              </Checkbox>
            </div>
          </DialogFormItem>
          <DialogFormItem title={t('shared.field.filter')} wrapperClassName={cx(flex_col, gap_y_1)}>
            <Select
              value={filterType}
              onChange={(e) => {
                if (typeof e === 'string') {
                  setFilterType(e)
                }
              }}
            >
              <Select.Option value={'none'}>{t('advancedDialogs.import.filterNone')}</Select.Option>
              <Select.Option value={'oneByOne'}>
                {t('advancedDialogs.import.filterOneByOne')}
              </Select.Option>
              <Select.Option value={'lines'}>
                {t('advancedDialogs.import.filterLines')}
              </Select.Option>
            </Select>
            <PureTextArea
              className={cx(filterType !== 'lines' && hidden)}
              value={filters}
              onChange={setFilters}
            ></PureTextArea>
          </DialogFormItem>
        </div>
      </ScrollArea>
    </Dialog>
  )
}
