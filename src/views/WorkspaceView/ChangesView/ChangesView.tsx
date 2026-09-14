import { Divider, Toast, Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { styled } from '@linaria/react'
import { writeText } from '@tauri-apps/plugin-clipboard-manager'
import { lstat, readFile, stat } from '@tauri-apps/plugin-fs'
import { useMemoizedFn } from 'ahooks'
import { useRef, useState } from 'react'
import { Group, Separator, Panel } from 'react-resizable-panels'

import { CatOptions } from '@/bindings/CatOptions'
import { nodePropertyNames } from '@/bindings/NodePropertyName'
import { PropertyListEntry } from '@/bindings/PropertyListEntry'
import { PropertyListOptions } from '@/bindings/PropertyListOptions'
import { PropertySetOptions } from '@/bindings/PropertySetOptions'
import { StatusEntry } from '@/bindings/StatusEntry'
import { StatusOptions } from '@/bindings/StatusOptions'
import { WcReplacedNode } from '@/bindings/WcReplacedNode'
import { ContextMenuItemModel } from '@/components/ContextMenu'
import { BinaryFile } from '@/components/DifferenceEditor'
import PureAutoComplete from '@/components/PureAutoComplete'
import PureTextArea from '@/components/PureTextArea'
import StrongDifferenceEditor, {
  IDifferenceTarget,
  StrongDifferenceEditorRef,
} from '@/components/StrongDifferenceEditor'
import { fsReadLink } from '@/context/Functions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useCurrentModal, useModal } from '@/lib/multi-modal'
import { localPath } from '@/utils/Path'
import ConfirmDialog from '@/views/dialogs/ConfirmDialog'
import { Dialog } from '@/views/dialogs/Dialog'
import DialogFormItem from '@/views/dialogs/DialogFormItem'

import Container from '../../../components/Container'
import FolderTreeIcon from '../../../icons/FolderTree.svg?react'
import { IconButton } from '../../../icons/IconButton'
import ListIcon from '../../../icons/List.svg?react'
import {
  min_h_0,
  min_w_0,
  m_1,
  flex,
  flex_1,
  flex_col,
  relative,
  absolute,
  inset_0,
  border_box,
  items_center,
  gap_x_1,
  overflow_hidden,
  gap_y_3,
} from '../../../styles/Classes'
import { ChangesListView } from './ChangesListView/ChangesListView'
import { ChangesTreeView } from './ChangesTreeView/ChangesTreeView'

const Div = styled.div`
  > svg {
    width: 20px;
    height: 20px;
  }
`

export interface ChangesViewProps {
  className?: string
}

function DeletePropertyDialog(props: { target: string; name: string }) {
  const modal = useCurrentModal()
  const subversion = useSubversion()
  const onOk = async () => {
    const options: PropertySetOptions = {
      local: {
        name: props.name,
        value: null,
        targets: [props.target],
        depth: 'empty',
        skipChecks: false,
        changelists: null,
      },
    }
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        await context.propertySet(options)
      },
    })
    modal.resolve(true)
    modal.hide()
  }
  return (
    <Dialog
      onOk={onOk}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      afterClose={modal.remove}
      visible={modal.visible}
    >
      <div className={cx(flex_1)}>
        <span>{`Are yout sure to delete property`}</span>
        <Typography.Text style={{ display: 'inline-block' }} code>
          {props.name}
        </Typography.Text>
        <span>{` on ${props.target} ?`}</span>
      </div>
    </Dialog>
  )
}

function EditPropertyDialog(props: {
  target: string
  name?: string
  value?: string
  allowEditName: boolean
  title?: string
}) {
  const modal = useCurrentModal()
  const subversion = useSubversion()
  const [name, setName] = useState(props.name ?? '')
  const [value, setValue] = useState(props.value ?? '')
  const onOk = async () => {
    if (name === '') {
      Toast.error({
        content: 'Property name must not be empty',
        stack: true,
      })
      return
    }
    const options: PropertySetOptions = {
      local: {
        name,
        value,
        targets: [props.target],
        depth: 'empty',
        skipChecks: false,
        changelists: null,
      },
    }
    await Subversion.callOnce({
      factory: subversion,
      async call(context) {
        await context.propertySet(options)
      },
    })
    modal.resolve(true)
    modal.hide()
  }
  const [names, setNames] = useState(nodePropertyNames)
  const onNameSearch = (value: string) => {
    setNames(nodePropertyNames.filter((i) => i.indexOf(value) > 0))
  }
  return (
    <Dialog
      title={props.title}
      onCancel={() => {
        modal.resolve(false)
        modal.hide()
      }}
      onOk={onOk}
      afterClose={modal.remove}
      visible={modal.visible}
    >
      <div className={cx(flex_1, flex, flex_col, gap_y_3)}>
        <DialogFormItem title="Name:" wrapperClassName={cx(flex)}>
          <PureAutoComplete
            data={names}
            onSearch={onNameSearch}
            className={cx(flex_1)}
            disabled={!props.allowEditName}
            value={name}
            onChange={(e) => {
              if (typeof e === 'string') {
                setName(e)
              }
            }}
          ></PureAutoComplete>
        </DialogFormItem>
        <DialogFormItem wrapperClassName={cx(flex)} title="Value:">
          <PureTextArea
            className={cx(flex_1)}
            value={value}
            onChange={(e) => {
              if (typeof e === 'string') {
                setValue(e)
              }
            }}
          ></PureTextArea>
        </DialogFormItem>
      </div>
    </Dialog>
  )
}

export function ChangesView({ className }: ChangesViewProps) {
  const [isListView, setIsListView] = useState(true)
  const [currentEditor, setCurrentEditor] = useState<string | null>(null)
  const [optionBarContainer, setOptionBarContainer] = useState<HTMLDivElement | null>(null)
  const subversion = useSubversion()
  const editor = useRef<StrongDifferenceEditorRef>(null)

  const modal = useModal()

  const onSelected = useMemoizedFn(async (path: string | null) => {
    if (path === null) {
      setCurrentEditor(null)
      return
    }

    const updateEntry = async () => {
      return await Subversion.call<{ entry: StatusEntry } | { error: any }>({
        factory: subversion,
        async call(context) {
          const options: StatusOptions = {
            path: path,
            revision: 'working',
            depth: 'empty',
            getAll: true,
            checkOutOfDate: false,
            checkWorkingCopy: false,
            noIgnore: true,
            ignoreExternals: false,
            depthAsSticky: false,
            changelist: null,
          }
          const result = (await context.status(options)).entries[0]
          if (result === undefined) {
            throw new Error(`'${path}' is not under version control`)
          }
          return {
            entry: result,
          }
        },
        onError(error) {
          return {
            error,
          }
        },
      })
    }

    const target: IDifferenceTarget = {
      loadNewContent: async function (
        expandKeywords: boolean,
      ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> {
        const result = await updateEntry()
        if ('error' in result) {
          return result
        }
        const entry = result.entry
        if (entry.nodeStatus === 'deleted' || entry.nodeStatus === 'missing') {
          return null
        }
        if (entry.nodeKind !== 'file' && entry.nodeKind !== 'symlink') {
          return null
        }
        const decoder = new TextDecoder('utf-8', { fatal: true })

        // 软链接按“链接目标”文本处理（与 svn 对 special file 的 normal form 一致）：
        // readFile 会跟随链接读目标内容，cat(working) 同样跟随链接，都不符合仓库实际存储的内容
        const metadata = await lstat(entry.path)
        if (metadata.isSymlink) {
          const target = await fsReadLink(entry.path)
          return {
            text: `link ${target}`,
          }
        }

        if (entry.nodeStatus === 'unversioned' || entry.nodeStatus === 'added') {
          const text = await readFile(entry.path)
          try {
            return {
              text: decoder.decode(text),
            }
          } catch {
            const status = await stat(entry.path)
            return {
              file: {
                name: localPath.getFileName(entry.path) ?? '',
                size: status.size,
              },
            }
          }
        }
        const options: CatOptions = {
          path: entry.path,
          pegRevision: 'base',
          revision: 'working',
          expandKeywords,
          getProperties: false,
        }
        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewContent']>>>({
          factory: subversion,
          async call(context) {
            const result = await context.cat(options)
            try {
              return {
                text: decoder.decode(result.content),
              }
            } catch {
              const status = await stat(entry.path)
              return {
                file: {
                  name: localPath.getFileName(entry.path) ?? '',
                  size: status.size,
                },
              }
            }
          },
          onError(error) {
            console.warn('on error', error)
            return {
              error,
            }
          },
        })
      },
      loadOldContent: async function (
        expandKeywords: boolean,
      ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> {
        const result = await updateEntry()

        if ('error' in result) {
          return result
        }
        const entry = result.entry

        if (entry.nodeStatus === 'unversioned' || entry.nodeStatus === 'added') {
          return null
        }

        const decoder = new TextDecoder('utf-8', { fatal: true })
        if (entry.nodeStatus === 'replaced') {
          // TODO: handle expandKeywords
          const node = await Subversion.call<{ node: WcReplacedNode | null } | { error: any }>({
            factory: subversion,
            async call(context) {
              const node = await context.wcGetReplacedFile(entry.path)
              return {
                node,
              }
            },
            onError(error) {
              return {
                error,
              }
            },
          })

          if ('error' in node) {
            return {
              error: node.error,
            }
          }

          if (node.node === null) {
            return null
          }

          if ('directory' in node.node) {
            return null
          }
          // return {
          //   text: decoder.decode(node.node.file.content),
          // }
          try {
            return {
              text: decoder.decode(node.node.file.content),
            }
          } catch {
            return {
              file: {
                name: localPath.getFileName(entry.path) ?? '',
                size: node.node.file.content.byteLength,
              },
            }
          }
        }

        if (entry.nodeKind !== 'file') {
          return null
        }

        const options: CatOptions = {
          path: entry.path,
          pegRevision: 'base',
          revision: 'base',
          expandKeywords,
          getProperties: false,
        }

        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadOldContent']>>>({
          factory: subversion,
          async call(context) {
            const result = await context.cat(options)
            try {
              return {
                text: decoder.decode(result.content),
              }
            } catch {
              return {
                file: {
                  name: localPath.getFileName(entry.path) ?? '',
                  size: result.content.byteLength,
                },
              }
            }
          },
          onError(error) {
            return {
              error,
            }
          },
        })
      },
      loadNewProperty: async function (): Promise<
        { property: PropertyListEntry } | { error: any } | null
      > {
        const result = await updateEntry()

        if ('error' in result) {
          return result
        }
        const entry = result.entry

        if (
          entry.nodeStatus === 'deleted' ||
          entry.nodeStatus === 'missing' ||
          entry.nodeStatus === 'unversioned'
        ) {
          return null
        }
        const options: PropertyListOptions = {
          target: entry.path,
          pegRevision: 'base',
          revision: 'working',
          depth: 'empty',
          changelists: null,
          inherited: false,
        }
        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewProperty']>>>({
          factory: subversion,
          async call(context) {
            const result = await context.propertyList(options)
            return {
              property: result.entries[0],
            }
          },
          onError(error) {
            return { error }
          },
        })
      },
      loadOldProperty: async function (): Promise<
        { property: PropertyListEntry } | { error: any } | null
      > {
        const result = await updateEntry()

        if ('error' in result) {
          return result
        }
        const entry = result.entry

        if (entry.nodeStatus === 'added' || entry.nodeStatus === 'unversioned') {
          return null
        }

        if (entry.nodeStatus === 'replaced') {
          const node = await Subversion.call<{ node: WcReplacedNode | null } | { error: any }>({
            factory: subversion,
            async call(context) {
              const node = await context.wcGetReplacedFile(entry.path)
              return {
                node,
              }
            },
            onError(error) {
              return {
                error,
              }
            },
          })

          if ('error' in node) {
            return { error: node.error }
          } else {
            if (node.node === null) {
              return null
            }
            if ('directory' in node.node) {
              return null
            } else {
              return {
                property: {
                  path: entry.path,
                  properties: node.node.file.originalProperties,
                  inheritedProperties: null,
                },
              }
            }
          }
        }

        const options: PropertyListOptions = {
          target: entry.path,
          pegRevision: 'base',
          revision: 'base',
          depth: 'empty',
          changelists: null,
          inherited: false,
        }

        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadOldProperty']>>>({
          factory: subversion,
          async call(context) {
            const result = await context.propertyList(options)
            return {
              property: result.entries[0],
            }
          },
          onError(error) {
            return { error }
          },
        })
      },
      propertyRowContextMenu: (context) => {
        return (row) => {
          const items: ContextMenuItemModel[] = []
          items.push({
            item: {
              content: 'Copy property name',
              onSelect: () => {
                writeText(row.original.name)
              },
            },
          })
          items.push({
            item: {
              content: 'Copy orignal property value',
              onSelect: () => {
                if (row.original.oldValue) {
                  writeText(row.original.oldValue)
                }
              },
            },
          })
          items.push({
            item: {
              content: 'Copy modified property value',
              onSelect: () => {
                if (row.original.newValue) {
                  writeText(row.original.newValue)
                }
              },
            },
          })
          items.push({
            separator: {},
          })
          items.push({
            item: {
              content: 'Edit',
              onSelect: async () => {
                const data = await updateEntry()
                if ('error' in data) {
                  Toast.error({
                    content: `Unexpected error ${data.error}`,
                    stack: true,
                  })
                  return
                }
                const result = await modal
                  .show(EditPropertyDialog, {
                    target: data.entry.path,
                    name: row.original.name,
                    value: row.original.newValue,
                    allowEditName: false,
                  })
                  .as<boolean>()
                if (result) {
                  context.update()
                }
              },
            },
          })
          if (row.original.status !== 'deleted') {
            items.push({
              item: {
                content: 'Delete',
                onSelect: async () => {
                  const data = await updateEntry()
                  if ('error' in data) {
                    Toast.error({
                      content: `Unexpected error ${data.error}`,
                      stack: true,
                    })
                    return
                  }
                  const result = await modal
                    .show(DeletePropertyDialog, {
                      target: data.entry.path,
                      name: row.original.name,
                    })
                    .as<boolean>()
                  if (result) {
                    await context.update()
                  }
                },
              },
            })
          }
          if (row.original.status !== 'unchanged') {
            items.push({
              item: {
                content: 'Revert',
                onSelect: async () => {
                  const data = await updateEntry()
                  if ('error' in data) {
                    Toast.error({
                      content: `Unexpected error ${data.error}`,
                      stack: true,
                    })
                    return
                  }

                  const result = await modal
                    .show(ConfirmDialog, {
                      title: '',
                      children: `Are you sure to revert property ${row.original.name}`,
                    })
                    .as<boolean>()

                  if (!result) {
                    return
                  }

                  if (data.entry.nodeStatus === 'deleted') {
                    return
                  }
                  let value: string | null = null
                  if (row.original.status === 'deleted') {
                    value = row.original.oldValue ?? null
                  } else if (row.original.status === 'added') {
                    value = null
                  } else if (row.original.status === 'modified') {
                    value = row.original.oldValue ?? ''
                  } else {
                    return
                  }
                  const options: PropertySetOptions = {
                    local: {
                      name: row.original.name,
                      value,
                      targets: [data.entry.path],
                      depth: 'empty',
                      skipChecks: false,
                      changelists: null,
                    },
                  }

                  await Subversion.callOnce({
                    factory: subversion,
                    async call(c) {
                      await c.propertySet(options)
                      context.update()
                    },
                  })
                },
              },
            })
          }

          return items
        }
      },
      addProperty: (context) => {
        return async () => {
          const data = await updateEntry()
          if ('error' in data) {
            Toast.error({
              content: `Unexpected error ${data.error}`,
              stack: true,
            })
            return
          }
          const result = await modal
            .show(EditPropertyDialog, {
              target: data.entry.path,
              allowEditName: true,
              title: 'Add property',
            })
            .as<boolean>()
          if (result) {
            await context.update()
          }
        }
      },
    }
    editor.current?.add(path, target)

    setCurrentEditor(path)
  })

  const refresh = () => {
    editor?.current?.deactivate()
  }

  return (
    <div className={cx(flex, min_w_0, className)}>
      <Group orientation="horizontal" className={cx(flex_1)}>
        <Panel className={cx(min_h_0, min_w_0, relative)} defaultSize={'30%'}>
          <div className={cx(min_h_0, min_w_0, flex, flex_col, absolute, inset_0)}>
            <Div className={cx(m_1, flex, gap_x_1, min_w_0, items_center, overflow_hidden)}>
              <div ref={setOptionBarContainer} className={cx(flex, flex_1)}></div>
              <Divider layout="vertical"></Divider>
              <IconButton
                color={isListView ? 'var(--semi-color-primary)' : undefined}
                onClick={() => setIsListView(true)}
              >
                <ListIcon></ListIcon>
              </IconButton>
              <IconButton
                color={!isListView ? 'var(--semi-color-primary)' : undefined}
                onClick={() => setIsListView(false)}
              >
                <FolderTreeIcon></FolderTreeIcon>
              </IconButton>
            </Div>
            <Divider></Divider>
            <Container className={cx(flex_1, min_w_0, min_h_0, flex, flex_col)}>
              <ChangesListView
                onRefresh={refresh}
                onSelected={onSelected}
                visible={isListView}
              ></ChangesListView>
              <ChangesTreeView
                onRefresh={refresh}
                onSelected={onSelected}
                optionBarContainer={optionBarContainer}
                visible={!isListView}
              ></ChangesTreeView>
            </Container>
          </div>
        </Panel>
        <Separator style={{ width: 5 }}></Separator>
        <Panel className={cx(flex)}>
          <div
            className={cx(
              flex_1,
              flex,
              border_box,
              min_w_0,
              css`
                padding-top: 5px;
              `,
            )}
          >
            <StrongDifferenceEditor
              className={cx(flex_1)}
              currentKey={currentEditor ?? undefined}
              ref={editor}
            ></StrongDifferenceEditor>
          </div>
        </Panel>
      </Group>
    </div>
  )
}
