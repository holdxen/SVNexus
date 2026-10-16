import { Toast, Typography } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { WebviewWindow } from '@tauri-apps/api/webviewWindow'
import { save } from '@tauri-apps/plugin-dialog'
import { writeFile } from '@tauri-apps/plugin-fs'
import { useEffect, useRef, useState } from 'react'
import { Group, Panel, Separator } from 'react-resizable-panels'
import * as uuid from 'uuid'

import { CatOptions } from '@/bindings/CatOptions'
import { LogChangedPathEntry } from '@/bindings/LogChangedPathEntry'
import { PropertyListEntry } from '@/bindings/PropertyListEntry'
import { PropertyListOptions } from '@/bindings/PropertyListOptions'
import { ContextMenu, ContextMenuItemModel } from '@/components/ContextMenu'
import { BinaryFile } from '@/components/DifferenceEditor'
import StrongDifferenceEditor, {
  IDifferenceTarget,
  StrongDifferenceEditorRef,
} from '@/components/StrongDifferenceEditor'
import ChangeActionIcon from '@/components/subversion/ChangeActionIcon'
import FileKindIcon from '@/components/subversion/FileKindIcon'
import VirtualList from '@/components/VirtualList'
import { base64Encode } from '@/context/Functions'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useT } from '@/i18n'
import {
  border_box,
  flex,
  flex_1,
  gap_x_1,
  items_center,
  min_h_0,
  min_w_0,
  overflow_hidden,
  p_1,
  px_1,
  py_1,
  whitespace_nowrap,
} from '@/styles/Classes'
import { list_item, list_item_selected } from '@/styles/Components'
import errorHumanString, { getSubversionError } from '@/utils/Error'
import Logger from '@/utils/Logger'
import simplifyPath, { repoPath } from '@/utils/Path'
import { combineUrl } from '@/utils/Url'

export interface ChangesViewProps {
  className?: string
  repositoryRoot: string
  relativePath: string
  currentRevision: number
  compareRevision: number
  entries: Record<string, LogChangedPathEntry>
}

const icon = css`
  svg {
    width: 20px;
    height: 20px;
    flex-shrink: 0;
  }
`

export function ChangesView(props: ChangesViewProps) {
  const entries = Object.entries(props.entries)

  const [selected, setSelected] = useState<number | null>(null)
  const editor = useRef<StrongDifferenceEditorRef | null>(null)
  const subversion = useSubversion()
  const t = useT()

  useEffect(() => {
    if (selected === null) {
      return
    }

    const [path, entry] = entries[selected]
    // const target: IDifferenceTarget = {
    //   target: props.repositoryRoot + path,
    //   pegRevision: { number: props.currentRevision },
    //   nodeKind: entry.nodeKind
    // }

    // if (entry.action === 'add') {
    //   target.currentRevision = { number: props.currentRevision }
    // } else if (entry.action === 'delete') {
    //   target.compareRevision = { number: props.compareRevision }
    //   target.pegRevision = { number: props.currentRevision - 1 }
    // } else {
    //   target.currentRevision = { number: props.currentRevision }
    //   target.compareRevision = { number: props.compareRevision }
    // }

    // target.hasProperty = true
    // target.hasContent = entry.nodeKind !== 'directory'
    //
    const target: IDifferenceTarget = {
      loadNewContent: async function (
        expandKeywords: boolean,
      ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> {
        if (entry.nodeKind !== 'file' && entry.action !== 'replace') return null
        if (entry.action === 'delete') return null

        const options: CatOptions = {
          path: props.repositoryRoot + path,
          pegRevision: { number: props.currentRevision },
          revision: { number: props.currentRevision },
          expandKeywords,
          getProperties: false,
        }

        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewContent']>>>({
          factory: subversion,
          call: async (context) => {
            const newContent = (await context.cat(options)).content
            const decoder = new TextDecoder('utf-8', { fatal: true })
            try {
              return { text: decoder.decode(newContent) }
            } catch {
              return {
                file: {
                  name: repoPath.getFileName(path) ?? '',
                  size: newContent.byteLength,
                },
              }
            }
          },
          onError: (error) => {
            const subversion = getSubversionError(error)
            if (subversion && subversion.code === 'clientIsDirectory') {
              return null
            }
            return {
              error,
            }
          },
        })
      },
      loadOldContent: async function (
        expandKeywords: boolean,
      ): Promise<{ text: string } | { file: BinaryFile } | { error: any } | null> {
        if (entry.nodeKind !== 'file' && entry.action !== 'replace') return null
        if (entry.action === 'add') return null

        const options: CatOptions = {
          path: props.repositoryRoot + path,
          pegRevision: {
            number:
              entry.action === 'delete' || entry.action === 'replace'
                ? props.compareRevision
                : props.currentRevision,
          },
          revision: { number: props.compareRevision },
          expandKeywords,
          getProperties: false,
        }

        return await Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadOldContent']>>>({
          factory: subversion,
          call: async (context) => {
            const oldContent = (await context.cat(options)).content
            const decoder = new TextDecoder('utf-8', { fatal: true })
            try {
              return { text: decoder.decode(oldContent) }
            } catch {
              return {
                file: {
                  name: repoPath.getFileName(path) ?? '',
                  size: oldContent.byteLength,
                },
              }
            }
          },
          onError: (error) => {
            const subversion = getSubversionError(error)
            if (subversion && subversion.code === 'clientIsDirectory') {
              return null
            }
            return {
              error,
            }
          },
        })
      },
      loadNewProperty: async function (): Promise<
        { property: PropertyListEntry } | { error: any } | null
      > {
        if (entry.action === 'delete') {
          return null
        }
        return Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadNewProperty']>>>({
          factory: subversion,
          call: async (context) => {
            const options: PropertyListOptions = {
              target: props.repositoryRoot + path,
              pegRevision: { number: props.currentRevision },
              revision: { number: props.currentRevision },
              depth: 'empty',
              changelists: null,
              inherited: false,
            }

            const result = await context.propertyList(options)
            return {
              property: result.entries[0],
            }
          },
          onError: (error) => {
            return {
              error,
            }
          },
        })
      },
      loadOldProperty: async function (): Promise<
        { property: PropertyListEntry } | { error: any } | null
      > {
        if (entry.action === 'add') {
          return null
        }
        return Subversion.call<Awaited<ReturnType<IDifferenceTarget['loadOldProperty']>>>({
          factory: subversion,
          call: async (context) => {
            const options: PropertyListOptions = {
              target: props.repositoryRoot + path,
              pegRevision: { number: props.compareRevision },
              revision: { number: props.compareRevision },
              depth: 'empty',
              changelists: null,
              inherited: false,
            }

            const result = await context.propertyList(options)
            return {
              property: result.entries[0],
            }
          },
          onError: (error) => {
            return {
              error,
            }
          },
        })
      },
    }

    editor.current?.add(path, target)

    // editor.current?.add(path, async () => {
    //   return (
    //     (await Subversion.callOnce({
    //       factory: subversion,
    //       call: async (context) => {
    //         const options: CatOptions = {
    //           path: props.repositoryRoot + path,
    //           pegRevision: { number: props.currentRevision },
    //           revision: { number: props.currentRevision },
    //           expandKeywords: true,
    //           getProperties: false,
    //         }

    //         const model: DifferenceModel = {}
    //         const decoder = new TextDecoder('utf-8', { fatal: true })

    //         if (entry.nodeKind === 'file') {
    //           if (entry.action === 'add') {
    //             const newContent = (await context.cat(options)).content
    //             model.newContent = decoder.decode(newContent)
    //           } else if (entry.action === 'delete') {
    //             const oldContent = (
    //               await context.cat({ ...options, revision: { number: props.compareRevision } })
    //             ).content
    //             model.oldContent = decoder.decode(oldContent)
    //           } else {
    //             const newContent = (await context.cat(options)).content
    //             const oldContent = (
    //               await context.cat({ ...options, revision: { number: props.compareRevision } })
    //             ).content
    //             model.newContent = decoder.decode(newContent)
    //             model.oldContent = decoder.decode(oldContent)
    //           }
    //         }

    //         return model
    //       },
    //     })) ?? {}
    //   )
    // })
  }, [selected])

  return (
    <div className={cx(min_w_0, min_h_0, border_box, p_1, props.className)}>
      <Group orientation="horizontal" className={cx(min_h_0)}>
        <Panel className={cx(flex, min_h_0)} defaultSize={'30%'}>
          <VirtualList
            className={cx(flex_1, min_w_0, min_h_0)}
            itemRender={(row) => {
              const [path, entry] = entries[row.index]
              let displayPath = simplifyPath(props.relativePath, path)

              let fileName
              let relativeDirectory

              if (displayPath === '') {
                fileName = repoPath.getFileName(path) ?? '/'
                relativeDirectory = ''
              } else {
                fileName = repoPath.getFileName(displayPath) ?? displayPath
                relativeDirectory = '/' + repoPath.getParent(displayPath)
                // if (relativeDirectory === '') {
                //   relativeDirectory = '/'
                // }
              }

              Logger.info('display path:', {
                path,
                relativePath: props.relativePath,
                displayPath,
                entry,
              })

              // 只有文件行才有菜单：目录没有可保存的内容，也没有文件历史
              const menu: ContextMenuItemModel[] = []

              if (entry.nodeKind === 'file') {
                // 被删除的文件在 currentRevision 已经不存在，内容得回到上一版取
                const revision =
                  entry.action === 'delete' ? props.compareRevision : props.currentRevision
                const url = combineUrl(props.repositoryRoot, path)

                menu.push({
                  item: {
                    content: t('shared.action.save'),
                    onSelect: async () => {
                      const target = await save({
                        title: t('dialogs.saveFile'),
                        defaultPath: fileName,
                        canCreateDirectories: true,
                      })
                      if (target === null) {
                        return
                      }

                      await Subversion.callOnce({
                        factory: subversion,
                        call: async (context) => {
                          const options: CatOptions = {
                            path: props.repositoryRoot + path,
                            pegRevision: { number: revision },
                            revision: { number: revision },
                            expandKeywords: false,
                            getProperties: false,
                          }

                          const content = await context.cat(options)
                          await writeFile(target, content.content)
                          Toast.success({
                            content: t('changes.history.saveSuccess', { path: target }),
                            stack: true,
                          })
                        },
                        onError: (error) => {
                          Toast.error({
                            content: t('changes.history.saveFailed', {
                              path: target,
                              error: errorHumanString(error),
                            }),
                            stack: true,
                          })
                        },
                      })
                    },
                  },
                })

                menu.push({
                  item: {
                    content: t('shared.action.fileHistory'),
                    onSelect: async () => {
                      try {
                        const id = uuid.v4()
                        const encoded = await base64Encode(new TextEncoder().encode(url), false)
                        const webview = new WebviewWindow(id, {
                          url: `/FileHistoryView/${encoded}/${revision}`,
                          title: url,
                          width: 800,
                          height: 600,
                        })
                        await webview.show()
                      } catch (e) {
                        Logger.error('Failed to create file history window: ', e)
                      }
                    },
                  },
                })
              }

              return (
                <ContextMenu menu={menu} asChild>
                  <div
                    style={{ height: 30 }}
                    onClick={() => setSelected(row.index)}
                    onContextMenu={() => setSelected(row.index)}
                    className={cx(
                      flex_1,
                      flex,
                      icon,
                      py_1,
                      px_1,
                      overflow_hidden,
                      gap_x_1,
                      items_center,
                      list_item,
                      whitespace_nowrap,
                      border_box,
                      row.index == selected && list_item_selected,
                    )}
                  >
                    <ChangeActionIcon action={entry.action} />
                    <FileKindIcon kind={entry.nodeKind}></FileKindIcon>

                    <div className={cx(flex_1, flex, gap_x_1)}>
                      <Typography.Text delete={entry.action === 'delete'}>
                        {fileName}
                      </Typography.Text>
                      <Typography.Text type="quaternary">{relativeDirectory}</Typography.Text>
                    </div>
                  </div>
                </ContextMenu>
              )
            }}
            count={entries.length}
            itemHeight={30}
          ></VirtualList>
          {/*<ScrollArea
            className={cx(flex_1, min_w_0, min_h_0)}
            contentClassName={cx(flex, flex_col, min_w_0)}
          >
            {entries.map(([path, entry], index) => {
              let displayPath = simplifyPath(props.relativePath, path)
              if (displayPath === '') {
                displayPath = '/'
              }
              return (
                <div
                  onClick={() => setSelected(index)}
                  key={index}
                  className={cx(
                    flex,
                    icon,
                    py_1,
                    px_1,
                    overflow_hidden,
                    gap_x_1,
                    items_center,
                    list_item,
                    whitespace_nowrap,
                    index == selected && list_item_selected,
                  )}
                >
                  <ChangeActionIcon action={entry.action} />
                  <FileKindIcon kind={entry.nodeKind}></FileKindIcon>

                  <DisplayContent
                    content={displayPath}
                    isDelete={entry.action === 'delete'}
                  ></DisplayContent>
                </div>
              )
            })}
          </ScrollArea>*/}
        </Panel>
        <Separator style={{ width: 4 }}></Separator>
        <Panel className={cx(flex)}>
          <StrongDifferenceEditor
            ref={editor}
            currentKey={selected === null ? undefined : entries[selected][0]}
            className={cx(flex_1)}
          ></StrongDifferenceEditor>
        </Panel>
      </Group>
    </div>
  )
}
