import { DndContext, MouseSensor, useSensors, useSensor, DragEndEvent } from '@dnd-kit/core'
import { restrictToVerticalAxis } from '@dnd-kit/modifiers'
import { SortableContext, useSortable, verticalListSortingStrategy } from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { Card, Typography, Divider, List, Toast, Tag } from '@douyinfe/semi-ui'
import { cx } from '@linaria/core'
import React, { CSSProperties, useEffect, useRef, useState } from 'react'

import { StatusOptions } from '@/bindings/StatusOptions'
import { WorkspaceItem } from '@/bindings/WorkspaceItem'
import { ContextMenu } from '@/components/ContextMenu'
import PureInput from '@/components/PureInput'
import { ScrollArea } from '@/components/ScrollArea'
import { useDatabase } from '@/context/Database'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useTabContent } from '@/context/TabContent'
import { useTabManager } from '@/context/TabManager'
import { ModalProvider, useCurrentModal, useModal } from '@/lib/multi-modal'
import { list_item, list_item_selected } from '@/styles/Components'
import { workspaceItemIdentity, workspaceItemStar } from '@/utils/WorkspaceItem'

import { WorkspaceGroup } from '../../bindings/WorkspaceGroup'
import DesktopAddIcon from '../../icons/DesktopAdd.svg?react'
import { IconButton } from '../../icons/IconButton'
import {
  cursor_default,
  h_full,
  min_h_0,
  grid,
  mx_1,
  my_2,
  overflow_y_auto,
  m_1,
  items_center,
  py_1,
  p_2,
  grid_rows_auto_auto_1fr,
  my_1,
  flex_1,
  flex,
  flex_col,
  min_w_0,
  overflow_hidden,
  cursor_grabbing,
  border_box,
  hidden,
} from '../../styles/Classes'
import ConfirmDialog from '../dialogs/ConfirmDialog'
import { Dialog } from '../dialogs/Dialog'
import { NiceEditWorkspaceItemDialog } from '../dialogs/EditWorkspaceItemDialog'
import { SubversionProvider } from '../SubversionProvider'
import { CommandBar } from './CommandBar'
import { WorkspaceGroupView } from './WorkspaceGroupView'
import WorkspaceItemView from './WorkspaceItemView'

const { Title, Text } = Typography

function HeaderTitle({ children }: { children?: React.ReactNode }) {
  return (
    <Title
      className={cx(cursor_default, min_w_0, overflow_hidden)}
      heading={1}
      style={{ margin: '0' }}
    >
      {children}
    </Title>
  )
}

function AddGroupDialog() {
  const [name, setName] = useState('')
  const modal = useCurrentModal()
  const workspaceGroups = useDatabase((state) => state.workspaceGroups)

  return (
    <Dialog
      visible={modal.visible}
      title="Add group"
      afterClose={() => modal.remove()}
      onOk={() => {
        if (name === '') {
          Toast.error({
            content: 'Empty name is not allowed',
            stack: true,
          })
          return
        }
        if (workspaceGroups.findIndex((e) => e.name === name) >= 0) {
          Toast.error({
            content: 'Group already exists',
            stack: true,
          })
          return
        }
        modal.resolve(name)
        modal.hide()
      }}
      onCancel={() => {
        modal.resolve(null)
        modal.hide()
      }}
    >
      <PureInput value={name} onChange={setName}></PureInput>
    </Dialog>
  )
}

type NavigationKind = 'All' | 'Star' | 'Modified' | 'Conflicted' | 'Invalid'

type WorkspaceItemFilter = (item: WorkspaceItem) => boolean

function WorkspaceGroupListItem({
  item,
  isGroupSelected,
  modal,
  setSelected,
  deleteWorkspaceGroup,
}: {
  item: WorkspaceGroup
  isGroupSelected: (identity: string) => boolean
  modal: ReturnType<typeof useModal>
  setSelected: (value: { kind: NavigationKind } | { group: string }) => void
  deleteWorkspaceGroup: (identity: string) => Promise<void>
}) {
  const { attributes, listeners, setNodeRef, transform, transition, isDragging, isOver } =
    useSortable({
      id: item.identity,
    })

  const styles: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
    ...(isDragging ? { zIndex: 999, position: 'relative' } : {}),
  }

  return (
    <div
      ref={setNodeRef}
      {...listeners}
      {...attributes}
      style={styles}
      className={cx(
        flex,
        isDragging && 'isDragging',
        isOver && 'isOver',
        list_item,
        isGroupSelected(item.identity) && list_item_selected,
        isDragging && cursor_grabbing,
      )}
    >
      <List.Item className={cx(flex, flex_1)}>
        <WorkspaceGroupView
          className={cx(flex_1)}
          item={item}
          onDelete={async () => {
            const result = await modal
              .show(ConfirmDialog, {
                icon: 'warning',
                children: "This operation can't be undone",
                title: `Whether to delete ${item.name}`,
              })
              .as<boolean>()
            if (result) {
              await deleteWorkspaceGroup(item.identity)
            }
          }}
          onClick={() => {
            setSelected({
              group: item.identity,
            })
          }}
        ></WorkspaceGroupView>
      </List.Item>
    </div>
  )
}

function Navigation(props: { onFilterChange?: (filter: WorkspaceItemFilter) => void }) {
  const kinds: NavigationKind[] = ['All', 'Star', 'Modified', 'Conflicted', 'Invalid']

  const load = useDatabase((status) => status.load)

  // const [kind, setKind] = useState(kinds[0])

  // const [workspaceGroupIndex, setWorkspaceGroupIndex] = useState<number | null>(null)

  const workspaceItemStates = useDatabase((status) => status.workspaceItemStates)
  const workspaceItems = useDatabase((status) => status.workspaceItems)
  const workspaceGroups = useDatabase((status) => status.workspaceGroups)
  const addWorkspaceGroup = useDatabase((status) => status.addWorkspaceGroup)
  const deleteWorkspaceGroup = useDatabase((status) => status.deleteWorkspaceGroup)
  const moveWorkspaceGroup = useDatabase((status) => status.moveWorkspaceGroup)

  const [selected, setSelected] = useState<{ kind: NavigationKind } | { group: string }>({
    kind: 'All',
  })

  const filter = (item: WorkspaceItem) => {
    const identity = workspaceItemIdentity(item)
    if ('kind' in selected) {
      switch (selected.kind) {
        case 'All':
          return true
        case 'Star':
          return workspaceItemStar(item)
        case 'Modified':
          return workspaceItemStates.get(identity)?.isModified ?? false
        case 'Conflicted':
          return workspaceItemStates.get(identity)?.isConflicted ?? false
        case 'Invalid':
          return workspaceItemStates.get(identity)?.isInvalid ?? false
        default:
          return false
      }
    } else if ('group' in selected) {
      const group = workspaceGroups.find((e) => e.identity === selected.group)
      if (group) {
        return group.members.includes(identity)
      }
    }
    return false
  }

  useEffect(() => {
    props.onFilterChange?.(filter)
  }, [selected, workspaceGroups, workspaceItems, workspaceItemStates])

  useEffect(() => {
    load()
  }, [])

  const onAddGroup = async (name: string) => {
    try {
      await addWorkspaceGroup(name, [])
    } catch (error) {
      console.error('Failed to add group:', error)
    }
  }

  const modal = useModal()

  const isGroupSelected = (identity: string) => {
    return 'group' in selected && selected.group === identity
  }

  const sensors = useSensors(
    useSensor(MouseSensor, {
      activationConstraint: { distance: 1 },
    }),
  )

  const handleDragEnd = (event: DragEndEvent) => {
    const { active, over } = event
    const from = workspaceGroups.findIndex((i) => i.identity == active.id)
    const to = workspaceGroups.findIndex((i) => i.identity == over?.id)
    moveWorkspaceGroup(from, to)
  }

  const isKindSelected = (kind: NavigationKind) => {
    return 'kind' in selected && selected.kind === kind
  }

  return (
    <>
      <Card
        shadows="always"
        className={cx(h_full, min_h_0, cursor_default, flex, min_w_0)}
        bodyStyle={{
          flex: '1',
          display: 'flex',
          flexDirection: 'column',
          padding: '10px',
          cursor: 'default',
          minHeight: '0px',
          minWidth: 0,
        }}
      >
        <HeaderTitle>{'SVNexus'}</HeaderTitle>
        <Divider className={my_1}></Divider>
        <div className={cx(flex, flex_col, flex_1, min_h_0)}>
          <div className={cx(flex, flex_col)}>
            <Text className={cx(mx_1, my_2)} type="tertiary" size="small">
              Navigation
            </Text>
            <List
              className={cx(min_h_0, overflow_y_auto, flex_1)}
              dataSource={kinds}
              split={false}
              renderItem={(item) => {
                let count = 0
                if (item === 'All') {
                  count = workspaceItems.length
                } else if (item === 'Star') {
                  count = workspaceItems.filter((e) => workspaceItemStar(e)).length
                } else if (item === 'Modified') {
                  count = Array.from(workspaceItemStates.values()).filter(
                    (e) => e.isModified === true,
                  ).length
                } else if (item === 'Conflicted') {
                  count = Array.from(workspaceItemStates.values()).filter(
                    (e) => e.isConflicted === true,
                  ).length
                } else if (item === 'Invalid') {
                  count = Array.from(workspaceItemStates.values()).filter(
                    (e) => e.isInvalid === true,
                  ).length
                } else {
                  console.warn('Unexpected kind:', item)
                }
                return (
                  <List.Item
                    style={{ fontWeight: 400, fontSize: '16px', padding: '12px 14px' }}
                    onClick={() => {
                      setSelected({
                        kind: item,
                      })
                      // setKind(item)
                    }}
                    className={cx(list_item, isKindSelected(item) && list_item_selected, flex)}
                  >
                    <div>{item}</div>
                    <div className={cx(flex_1)}></div>
                    <Tag color="indigo" shape="circle">
                      {count}
                    </Tag>
                  </List.Item>
                )
              }}
            ></List>
          </div>
          <Divider className={my_1}></Divider>
          <div className={cx(flex, flex_col, min_h_0, flex_1)}>
            <div className={cx(m_1, flex, items_center)}>
              <Text type="tertiary" className={py_1} size="small">
                Group
              </Text>
              <div className={cx(flex_1)}></div>
              <IconButton
                onClick={async () => {
                  const name = await modal.show(AddGroupDialog, {}).as<string | null>()
                  if (name !== null) {
                    await onAddGroup(name)
                  }
                  // setAddGroupDialogVisible(true)
                }}
                size={20}
              >
                <DesktopAddIcon></DesktopAddIcon>
              </IconButton>
            </div>
            <DndContext
              autoScroll={true}
              sensors={sensors}
              modifiers={[restrictToVerticalAxis]}
              onDragEnd={handleDragEnd}
            >
              <SortableContext
                items={workspaceGroups.map((i) => i.identity)}
                strategy={verticalListSortingStrategy}
              >
                <ScrollArea className={cx(flex_1)}>
                  <List
                    className={`${min_h_0} ${overflow_y_auto}`}
                    dataSource={workspaceGroups}
                    split={false}
                    renderItem={(item) => (
                      <WorkspaceGroupListItem
                        key={item.identity}
                        item={item}
                        isGroupSelected={isGroupSelected}
                        modal={modal}
                        setSelected={setSelected}
                        deleteWorkspaceGroup={deleteWorkspaceGroup}
                      ></WorkspaceGroupListItem>
                    )}
                  ></List>
                </ScrollArea>
              </SortableContext>
            </DndContext>
          </div>
        </div>
      </Card>
    </>
  )
}

function WorkspaceItemModelView(props: { model: WorkspaceItem; className?: string }) {
  function workspaceItemName(item: WorkspaceItem): string {
    if ('workingCopy' in item) {
      return item.workingCopy.name
    } else {
      return item.repository.name
    }
  }

  const workspaceItemStates = useDatabase((state) => state.workspaceItemStates)
  const setWorkspaceItemState = useDatabase((state) => state.setWorkspaceItemState)
  const identity = workspaceItemIdentity(props.model)

  const subversion = useSubversion()

  // const tags: React.ReactNode[] = []
  //
  // const [tags, setTags] = useState<React.ReactNode[]>([])

  const item = useRef<HTMLDivElement>(null)

  const itemState = workspaceItemStates.get(identity)

  const tags: React.ReactNode[] = []

  if (itemState) {
    if (itemState.revision) {
      const revision = itemState.revision
      const text =
        revision.min === revision.max ? `r${revision.min}` : `r${revision.min}:r${revision.max}`
      const tag = (
        <Tag type="solid" shape="circle" color="purple">
          {text}
        </Tag>
      )
      tags.push(tag)
    }
    if (itemState.isInvalid === true) {
      const tag = (
        <Tag type="solid" shape="circle" color="purple">
          Invalid
        </Tag>
      )
      tags.push(tag)
    }
    if (itemState.isConflicted === true) {
      const tag = (
        <Tag type="solid" shape="circle" color="violet">
          Conflicted
        </Tag>
      )
      tags.push(tag)
    }
    if (itemState.isModified) {
      const tag = (
        <Tag type="solid" shape="circle" color="amber">
          Modified
        </Tag>
      )
      tags.push(tag)
    }
    if (itemState.isLocked === true) {
      const tag = (
        <Tag type="solid" shape="circle" color="red">
          Locked
        </Tag>
      )
      tags.push(tag)
    }
    if (itemState.isClean === true) {
      const tag = (
        <Tag type="solid" shape="circle" color="green">
          Clean
        </Tag>
      )
      tags.push(tag)
    }
  }

  const updateTags = async () => {
    if ('workingCopy' in props.model) {
      const workingCopy = props.model.workingCopy
      await Subversion.callOnce({
        factory: subversion,
        async call(context) {
          const result = await context.wcRevisionStatus({
            localAbsolutePath: workingCopy.workingCopyRoot,
            trailUrl: null,
            committed: false,
          })

          const options: StatusOptions = {
            path: workingCopy.workingCopyRoot,
            revision: 'unspecified',
            depth: 'infinity',
            getAll: false,
            checkOutOfDate: false,
            checkWorkingCopy: false,
            noIgnore: false,
            ignoreExternals: false,
            depthAsSticky: false,
            changelist: null,
          }
          const entries = (await context.status(options)).entries

          let modified = false
          let conflicted = false
          let locked = false

          for (const entry of entries) {
            if (entry.wcIsLocked) {
              locked = true
            }
            if (entry.nodeStatus === 'modified') {
              modified = true
            }
            if (entry.nodeStatus === 'conflicted') {
              conflicted = true
            }
          }

          setWorkspaceItemState(identity, {
            revision: {
              min: result.status.minRevision,
              max: result.status.maxRevision,
            },
            isModified: modified,
            isConflicted: conflicted,
            isInvalid: false,
            isLocked: locked,
            isClean: entries.length === 0,
          })
        },
        onError(error) {
          console.warn('Working copy is invalid', error)
          setWorkspaceItemState(identity, {
            isInvalid: true,
          })
        },
      })
    }
  }

  useEffect(() => {
    if (workspaceItemStates.has(identity)) {
      return
    }
    if (item.current === null) {
      return
    }

    const element = item.current

    const observer = new IntersectionObserver(
      ([entry]) => {
        if (entry.isIntersecting) {
          updateTags()
          observer.unobserve(element)
        }
      },
      {
        threshold: 0.1,
      },
    )

    observer.observe(element)

    return () => {
      observer.disconnect()
    }
  }, [])

  const modal = useModal()

  const onEdit = () => {
    modal
      .show(NiceEditWorkspaceItemDialog, {
        item: identity,
      })
      .as<boolean>()
  }

  if ('workingCopy' in props.model) {
    return (
      <WorkspaceItemView
        ref={item}
        tags={tags}
        onEditClick={onEdit}
        className={props.className}
        name={workspaceItemName(props.model)}
        title1={props.model.workingCopy.workingCopyRoot}
        title2={''}
        pin={false}
        star={props.model.workingCopy.star}
      ></WorkspaceItemView>
    )
  } else {
    return (
      <WorkspaceItemView
        ref={item}
        className={props.className}
        name={workspaceItemName(props.model)}
        onEditClick={onEdit}
        title1={props.model.repository.repositoryRootUrl}
        title2={''}
        pin={false}
        star={props.model.repository.star}
      ></WorkspaceItemView>
    )
  }
}

function workspaceItemContainText(item: WorkspaceItem, text: string): boolean {
  if (text === '') {
    return true
  }
  if ('workingCopy' in item) {
    const workingCopy = item.workingCopy
    return (
      workingCopy.name.includes(text) ||
      workingCopy.workingCopyRoot.includes(text) ||
      workingCopy.workingCopyPath.includes(text)
    )
  } else {
    const repository = item.repository
    return repository.name.includes(text) || repository.repositoryRootUrl.includes(text)
  }
}

function WorkspaceItems(props: { filter?: WorkspaceItemFilter }) {
  const workspaceItems = useDatabase((state) => state.workspaceItems)
  const deleteWorkspaceItem = useDatabase((state) => state.deleteWorkspaceItem)
  const [selected, setSelected] = useState<string | null>(null)
  const [searchText, setSearchText] = useState('')
  const tabManager = useTabManager()
  const tabContent = useTabContent()

  const tryTodelete = async (item: WorkspaceItem) => {
    await deleteWorkspaceItem(workspaceItemIdentity(item))
  }

  const open = async (item: WorkspaceItem) => {
    if ('workingCopy' in item) {
      tabManager.openWorkingCopy(tabContent.identity, item.workingCopy.workingCopyPath)
    }
  }

  const filter = (item: WorkspaceItem) => {
    return (props.filter?.(item) ?? true) && workspaceItemContainText(item, searchText)
  }

  return (
    <Card
      shadows="always"
      // className="h-full min-h-0 cursor-default"
      className={cx(h_full, min_h_0, flex, cursor_default)}
      bodyStyle={{
        flex: '1',
        display: 'flex',
        flexDirection: 'column',
        padding: '10px',
        cursor: 'default',
        boxSizing: 'border-box',
        rowGap: 5,
        minWidth: 0,
      }}
    >
      <HeaderTitle>History</HeaderTitle>
      <Divider></Divider>
      <PureInput value={searchText} onChange={setSearchText}></PureInput>
      <ScrollArea
        className={cx(min_h_0, min_w_0, flex_1)}
        contentClassName={cx(flex, flex_col, min_w_0)}
      >
        {workspaceItems.map((e) => {
          const visible = filter(e)
          const identity = workspaceItemIdentity(e)
          return (
            <ContextMenu
              onDoubleClick={() => open(e)}
              onClick={() => setSelected(identity)}
              key={identity}
              className={cx(
                border_box,
                p_2,
                list_item,
                identity === selected && list_item_selected,
                flex,
                !visible && hidden,
                min_w_0,
              )}
              menu={[
                {
                  item: {
                    content: 'Open',
                    onSelect: () => {
                      open(e)
                    },
                  },
                },
                {
                  item: {
                    content: 'Delete',
                    onSelect: () => {
                      tryTodelete(e)
                    },
                  },
                },
              ]}
            >
              <WorkspaceItemModelView
                className={cx(flex_1, min_w_0, overflow_hidden)}
                model={e}
              ></WorkspaceItemModelView>
            </ContextMenu>
          )
        })}
      </ScrollArea>
    </Card>
  )
}

export function WelcomeView() {
  const [filter, setFilter] = useState<WorkspaceItemFilter>()
  return (
    <SubversionProvider className={h_full} singleton={false}>
      <ModalProvider>
        <div className={cx(grid, min_h_0, grid_rows_auto_auto_1fr, flex_1)}>
          <CommandBar></CommandBar>
          <Divider></Divider>
          <div
            className={cx(min_h_0, p_2)}
            style={{ display: 'grid', gridTemplateColumns: '1fr 2fr 1fr', gap: '8px' }}
          >
            <Navigation onFilterChange={(e) => setFilter(() => e)}></Navigation>
            <WorkspaceItems filter={filter}></WorkspaceItems>
            <Card shadows="always" className={cx(h_full)}></Card>
          </div>
        </div>
      </ModalProvider>
    </SubversionProvider>
  )
}
