import { Card, Divider, Tag, Tooltip } from '@douyinfe/semi-ui'
import AddPropertyIcon from '@icons/AddProperty.svg?react'
import ChangeOnlyIcon from '@icons/ChangeOnly.svg?react'
import ExpandKeywardsIcon from '@icons/ExpandKeywords.svg?react'
import FullscreenEnterIcon from '@icons/FullscreenEnter.svg?react'
import FullscreenExitIcon from '@icons/FullscreenExit.svg?react'
import PropertyViewIcon from '@icons/PropertyView.svg?react'
import SideBySideIcon from '@icons/SideBySide.svg?react'
import TextViewIcon from '@icons/TextView.svg?react'
import { css, cx } from '@linaria/core'
import { Row, type ColumnDef } from '@tanstack/react-table'
import { useMemoizedFn, useUpdateLayoutEffect } from 'ahooks'
import { RefObject, useEffect, useImperativeHandle, useMemo, useRef, useState } from 'react'
import { createPortal } from 'react-dom'
import { createHtmlPortalNode, InPortal, OutPortal } from 'react-reverse-portal'

import { PropertyListEntry } from '@/bindings/PropertyListEntry'
import { IconButton } from '@/icons/IconButton'
import {
  flex_1,
  flex,
  flex_col,
  border_box,
  min_w_0,
  min_h_0,
  hidden,
  m_2,
  absolute,
  inset_0,
} from '@/styles/Classes'
import errorHumanString from '@/utils/Error'
import { LimitedDictionary } from '@/utils/LimitedDictionary'
import { SingleTaskQueue } from '@/utils/Queue'
import { useChangedEffect } from '@/utils/React'
import { useModalDialog } from '@/views/DialogContext'

import { ContextMenuItemModel } from './ContextMenu'
import DifferenceEditor, { BinaryFile, DifferenceModel } from './DifferenceEditor'
import HoverTooltip from './HoverTooltip'
import LoadingLayer, { LoadingState } from './LoadingLayer'
import RadioIconGroup from './RadioIconGroup'
import { Table } from './Table'

export interface StrongDifferenceEditorProps {
  currentKey?: string
  className?: string
  ref?: RefObject<StrongDifferenceEditorRef | null>
}

type Property = PropertyListEntry

export interface IDifferenceTarget {
  loadNewContent: (
    expandKeywords: boolean,
  ) => Promise<{ text: string } | { file: BinaryFile } | { error: any } | null>
  loadOldContent: (
    expandKeywords: boolean,
  ) => Promise<{ text: string } | { file: BinaryFile } | { error: any } | null>

  loadNewProperty: () => Promise<{ property: Property } | { error: any } | null>
  loadOldProperty: () => Promise<{ property: Property } | { error: any } | null>
  propertyRowContextMenu?: (
    context: PropertyModalContext,
  ) => ((row: Row<PropertyRow>) => ContextMenuItemModel[]) | undefined
  addProperty?: (context: PropertyModalContext) => (() => void) | undefined
}

export interface StrongDifferenceEditorRef {
  add: (key: string, target: IDifferenceTarget) => void
  clear: () => void
  deactivate: () => void
}

export interface ErrorModel {
  message?: string
  retry: () => Promise<void>
}

export interface PropertyModel {
  newProperties?: PropertyListEntry
  oldProperties?: PropertyListEntry
}

export type PropertyStatus = 'added' | 'deleted' | 'modified' | 'unchanged'

export interface PropertyRow {
  name: string
  oldValue?: string
  newValue?: string
  status: PropertyStatus
}

function propertyStatusOf(
  oldValue: string | undefined,
  newValue: string | undefined,
): PropertyStatus {
  if (oldValue === undefined) {
    return 'added'
  }
  if (newValue === undefined) {
    return 'deleted'
  }
  return oldValue === newValue ? 'unchanged' : 'modified'
}

function propertyStatusTag(status: PropertyStatus) {
  switch (status) {
    case 'added':
      return (
        <Tag size="small" color="green">
          Added
        </Tag>
      )
    case 'deleted':
      return (
        <Tag size="small" color="red">
          Deleted
        </Tag>
      )
    case 'modified':
      return (
        <Tag size="small" color="amber">
          Modified
        </Tag>
      )
    case 'unchanged':
      return (
        <Tag size="small" color="grey">
          Unchanged
        </Tag>
      )
  }
}

const propertyColumns: ColumnDef<PropertyRow, any>[] = [
  { accessorKey: 'name', header: 'Name', size: 200 },
  {
    accessorKey: 'oldValue',
    header: 'Orignal Value',
    size: 200,
    cell: (info) => {
      return info.getValue() === undefined ? '' : info.getValue()
    },
  },
  {
    accessorKey: 'newValue',
    header: 'Modified Value',
    size: 200,
    cell: (info) => (info.getValue() === undefined ? '' : info.getValue()),
  },
  {
    id: 'status',
    header: 'Status',
    size: 100,
    cell: (info) => propertyStatusTag(info.row.original.status),
  },
]

// export interface StrongDiferenceModel {
//   content?: DifferenceModel | ErrorModel | null | Promise<DifferenceModel | ErrorModel>
//   property?: PropertyModel | ErrorModel | null | Promise<PropertyModel | ErrorModel>
// }

type ViewType = 'text' | 'property'

function errorModel<T>(state: State<T> | undefined): ErrorModel | undefined {
  if (typeof state === 'object') {
    if ('error' in state) {
      return state.error
    }
  }
  return undefined
}

function differenceModel(state: State<DifferenceModel> | undefined): DifferenceModel | undefined {
  if (typeof state === 'object') {
    if ('data' in state) {
      return state.data
    }
  }
  return undefined
}

export interface DifferenceContentProps {
  target?: IDifferenceTarget
  // view: ViewType
  visible: boolean
  expandKeywords: boolean
  showAllProperties: boolean
  sideBySide: boolean
  viewType: ViewType
  hideUnchanged: boolean
  className?: string

  iconContainer: HTMLDivElement | null

  // propertyRowContextMenu?: (
  //   context: PropertyModalContext,
  // ) => ((row: Row<PropertyRow>) => ContextMenuItemProps[]) | undefined
}

type State<T> =
  | {
      data: T
    }
  | { error: ErrorModel }
  | 'loading'

export interface PropertyModalContext {
  update: () => Promise<void>
}

function DifferenceContent(props: DifferenceContentProps) {
  const [content, setContent] = useState<State<DifferenceModel>>()
  const [property, setProperty] = useState<State<PropertyModel>>()

  const queue = useRef(new SingleTaskQueue())
  const loadedContent = useRef(false)
  const loadedProperty = useRef(false)

  const loadContent = useMemoizedFn(async (signal: AbortSignal) => {
    if (signal.aborted) return

    if (props.target === undefined) return

    let model: DifferenceModel = {
      newContent: undefined,
      oldContent: undefined,
    }
    let newContent = await props.target.loadNewContent(props.expandKeywords)
    if (newContent) {
      if ('error' in newContent) {
        setContent({
          error: {
            message: errorHumanString(newContent.error),
            retry: async () => queue.current.run(loadContent),
          },
        })
        return
      } else if ('text' in newContent) {
        model.newContent = newContent.text
      } else if ('file' in newContent) {
        model.newContent = newContent.file
      }
    }

    let oldContent = await props.target.loadOldContent(props.expandKeywords)
    if (oldContent) {
      if ('error' in oldContent) {
        setContent({
          error: {
            message: errorHumanString(oldContent.error),
            retry: async () => queue.current.run(loadContent),
          },
        })
        return
      } else if ('text' in oldContent) {
        model.oldContent = oldContent.text
      } else if ('file' in oldContent) {
        model.oldContent = oldContent.file
      }
    }
    setContent({
      data: model,
    })
  })

  const loadProperty = useMemoizedFn(async () => {
    const model: PropertyModel = {
      newProperties: undefined,
      oldProperties: undefined,
    }

    if (props.target === undefined) return

    const newProperties = await props.target.loadNewProperty()

    if (newProperties) {
      if ('error' in newProperties) {
        setProperty({
          error: {
            message: errorHumanString(newProperties.error),
            retry: loadProperty,
          },
        })
      } else if ('property' in newProperties) {
        model.newProperties = newProperties.property
      }
    }

    const oldProperties = await props.target.loadOldProperty()
    if (oldProperties) {
      if ('error' in oldProperties) {
        setProperty({
          error: {
            message: errorHumanString(oldProperties.error),
            retry: loadProperty,
          },
        })
      } else if ('property' in oldProperties) {
        model.oldProperties = oldProperties.property
      }
    }

    setProperty({
      data: model,
    })
  })

  useEffect(() => {
    if (!props.visible) {
      return
    }
    if (props.target === undefined) {
      setProperty(undefined)
      return
    }
    if (props.viewType !== 'property') {
      return
    }
    if (property !== undefined && loadedProperty.current) {
      return
    }
    setProperty('loading')
    loadedProperty.current = true
    loadProperty()
    // setProperty(() => loadProperty())
  }, [props.viewType, props.visible])

  useChangedEffect(
    ([expandKeywordsChanged]) => {
      if (!props.visible) {
        return
      }
      if (props.target === undefined) {
        setContent(undefined)
        return
      }
      if (props.viewType !== 'text') {
        return
      }
      // if (props.target.hasContent !== true) {
      //   setContent(undefined)
      //   return
      // }
      // if (props.target.nodeKind !== 'file') {
      //   setContent(undefined)
      //   return
      // }
      if (content !== undefined && expandKeywordsChanged === false && loadedContent.current) {
        return
      }

      setContent('loading')
      loadedContent.current = true
      queue.current.run(loadContent)
    },
    [props.expandKeywords, props.viewType, props.visible],
  )

  const state = (state: State<any> | undefined): LoadingState => {
    if (typeof state === 'undefined') {
      return 'none'
    }
    if (typeof state === 'string' && state === 'loading') {
      return 'loading'
    } else if ('error' in state) {
      return 'error'
    } else {
      return 'none'
    }
  }

  const propertyRows = useMemo(() => {
    const model = typeof property === 'object' && 'data' in property ? property.data : undefined
    const oldProperties = model?.oldProperties?.properties ?? {}
    const newProperties = model?.newProperties?.properties ?? {}
    const names = Array.from(
      new Set([...Object.keys(oldProperties), ...Object.keys(newProperties)]),
    ).sort()
    const rows: PropertyRow[] = names.map((name) => {
      const oldValue = oldProperties[name]
      const newValue = newProperties[name]
      return {
        name,
        oldValue,
        newValue,
        status: propertyStatusOf(oldValue, newValue),
      }
    })
    return props.hideUnchanged ? rows.filter((row) => row.status !== 'unchanged') : rows
  }, [property, props.hideUnchanged])

  // useEffect(() => {}, [content])

  // useEffect(() => {
  //   console.log('property state changed:', state(property), property)
  // }, [property])

  const addPropertyIcon = (
    <Tooltip content="Add property">
      <IconButton
        className={cx(
          (!props.visible ||
            props.target?.addProperty === undefined ||
            props.viewType !== 'property') &&
            hidden,
        )}
        onClick={() => {
          const call = props.target?.addProperty?.({
            update: loadProperty,
          })
          if (call) {
            call()
          }
        }}
      >
        <AddPropertyIcon></AddPropertyIcon>
      </IconButton>
    </Tooltip>
  )

  return (
    <div className={cx(!props.visible && hidden, min_w_0, min_h_0, flex, props.className)}>
      {props.iconContainer === null ? (
        <></>
      ) : (
        createPortal(addPropertyIcon, props.iconContainer, 'add proprty icon')
      )}
      <LoadingLayer
        errorMessage={errorModel(content)?.message}
        retry={errorModel(content)?.retry}
        state={state(content)}
        className={cx(props.viewType !== 'text' && hidden, flex_1)}
      >
        <DifferenceEditor
          newContent={differenceModel(content)?.newContent}
          oldContent={differenceModel(content)?.oldContent}
          // newContent={isDifferenceModel(content) ? content.newContent : undefined}
          // oldContent={isDifferenceModel(content) ? content.oldContent : undefined}
          className={cx(flex_1)}
          hideUnChanged={props.hideUnchanged}
          sideBySide={props.sideBySide}
        ></DifferenceEditor>
      </LoadingLayer>
      <LoadingLayer
        errorMessage={errorModel(property)?.message}
        retry={errorModel(property)?.retry}
        state={state(property)}
        className={cx(props.viewType !== 'property' && hidden, flex_1, min_w_0, min_h_0)}
        contentClassName={cx(min_w_0, min_h_0)}
      >
        <Table
          selectionMode="row"
          data={propertyRows}
          columns={propertyColumns}
          className={cx(flex_1, min_h_0, min_w_0)}
          onGetRowId={(row) => row.name}
          rowContextMenu={props.target?.propertyRowContextMenu?.({
            update: loadProperty,
          })}
        ></Table>
      </LoadingLayer>
    </div>
  )
}

export default function StrongDifferenceEditor({ ref, ...props }: StrongDifferenceEditorProps) {
  const [sideBySide, setSideBySide] = useState(false)
  const [currentViewType, setCurrentViewType] = useState<'text' | 'property'>('text')
  const [hideUnchanged, setHideUnchanged] = useState(false)
  const [editors, setEditors] = useState<LimitedDictionary<string, IDifferenceTarget>>(
    new LimitedDictionary(10),
  )
  // const subversion = useSubversion()
  const [expandKeywords, setExpandKeywords] = useState(false)
  const showAllProperties = !hideUnchanged
  const [version, setVersion] = useState(0)
  // const [showAllProperties, setShowAllProperties] = useState(false)

  useImperativeHandle(
    ref,
    () => ({
      add: (key, target) => {
        setEditors((v) => {
          v.add(key, target)
          return v.clone()
        })
      },
      clear: () => {
        setEditors(new LimitedDictionary(100))
      },

      deactivate: () => {
        setVersion((v) => v + 1)
      },
    }),
    [editors, props.currentKey],
  )

  // const loading = props.currentKey !== undefined && !editors.dictionary.has(props.currentKey)

  // const placeHolderVisible = props.currentKey === undefined || loading

  // const addProperty = props.currentKey
  //   ? editors.dictionary.get(props.currentKey)?.addProperty
  //   : undefined

  const [propertyIconContainer, setPropertyIconContainer] = useState<HTMLDivElement | null>(null)

  // const root = useRef(null)

  // const [isFullscreen, { toggleFullscreen } ] = useFullscreen(root, {
  //   pageFullscreen: {
  //     zIndex: 300
  //   }
  // })

  const portalNode = useMemo(
    () =>
      createHtmlPortalNode({
        attributes: {
          class: cx(
            flex,
            min_h_0,
            min_w_0,
            border_box,
            props.className,
            '________________________________portal',
          ),
        },
      }),
    [],
  )

  const [maximized, setMaximized] = useState(false)
  const modalDialog = useModalDialog()

  const zIndex = css`
    z-index: 300;
  `
  useUpdateLayoutEffect(() => {
    portalNode.element.className = cx(
      flex,
      min_h_0,
      min_w_0,
      border_box,
      props.className,
      '________________________________portal',
    )
    if (maximized) {
      portalNode.element.classList.add(absolute)
      portalNode.element.classList.add(inset_0)
      portalNode.element.classList.add(zIndex)
    } else {
      portalNode.element.classList.remove(absolute)
      portalNode.element.classList.remove(inset_0)
      portalNode.element.classList.remove(zIndex)
    }
  }, [maximized, props.className])

  return (
    <>
      <InPortal node={portalNode}>
        {/*<div ref={root} className={cx(flex, min_h_0, min_w_0, border_box, props.className)}>*/}
        <Card
          className={cx(flex, flex_col, border_box, min_w_0, flex_1, maximized && m_2)}
          bodyStyle={{
            display: 'flex',
            flex: '1',
            flexDirection: 'column',
            minWidth: 0,
            minHeight: 0,
          }}
        >
          <DifferenceContent
            key={0}
            className={cx(flex_1)}
            visible={props.currentKey === undefined}
            expandKeywords={expandKeywords}
            showAllProperties={showAllProperties}
            sideBySide={sideBySide}
            viewType={currentViewType}
            hideUnchanged={hideUnchanged}
            iconContainer={null}
          ></DifferenceContent>
          {Array.from(editors.dictionary.entries()).map(([key, target]) => {
            return (
              <DifferenceContent
                key={`${key}-${version}`}
                target={target}
                className={cx(flex_1)}
                visible={props.currentKey === key}
                expandKeywords={expandKeywords}
                showAllProperties={showAllProperties}
                sideBySide={sideBySide}
                viewType={currentViewType}
                hideUnchanged={hideUnchanged}
                iconContainer={propertyIconContainer}
              ></DifferenceContent>
            )
          })}
          <Divider></Divider>
          <div className={cx(flex)}>
            <RadioIconGroup
              active={currentViewType}
              icons={[
                {
                  icon: <TextViewIcon></TextViewIcon>,
                  tooltip: 'Text',
                  identity: 'text',
                  onClick() {
                    setCurrentViewType('text')
                  },
                },
                {
                  icon: <PropertyViewIcon></PropertyViewIcon>,
                  identity: 'property',
                  tooltip: 'Property',
                  onClick() {
                    setCurrentViewType('property')
                  },
                },
              ]}
            ></RadioIconGroup>
            <div className={cx(flex_1)}></div>
            <div ref={setPropertyIconContainer}></div>
            <HoverTooltip content={maximized ? 'Exit Fullscreen' : 'Enter Fullscreen'}>
              <IconButton onClick={() => setMaximized((v) => !v)}>
                {maximized ? (
                  <FullscreenExitIcon></FullscreenExitIcon>
                ) : (
                  <FullscreenEnterIcon></FullscreenEnterIcon>
                )}
              </IconButton>
            </HoverTooltip>
            <Tooltip content="Expand Keywords">
              <IconButton
                color={expandKeywords ? 'var(--semi-color-primary)' : undefined}
                onClick={() => setExpandKeywords((value) => !value)}
              >
                <ExpandKeywardsIcon></ExpandKeywardsIcon>
              </IconButton>
            </Tooltip>
            <Tooltip content="Hide unchanged">
              <IconButton
                color={hideUnchanged ? 'var(--semi-color-primary)' : undefined}
                onClick={() => setHideUnchanged((value) => !value)}
              >
                <ChangeOnlyIcon></ChangeOnlyIcon>
              </IconButton>
            </Tooltip>
            <Tooltip content="Side by Side">
              <IconButton
                color={sideBySide ? 'var(--semi-color-primary)' : undefined}
                onClick={() => setSideBySide((v) => !v)}
              >
                <SideBySideIcon></SideBySideIcon>
              </IconButton>
            </Tooltip>
          </div>
        </Card>
        {/*</div>*/}
      </InPortal>
      {!maximized && <OutPortal node={portalNode} />}
      {maximized && createPortal(<OutPortal node={portalNode} />, modalDialog.container())}
    </>
  )
}
