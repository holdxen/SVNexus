import {
  closestCenter,
  DndContext,
  type DragEndEvent,
  KeyboardSensor,
  PointerSensor,
  useSensor,
  useSensors,
} from '@dnd-kit/core'
import { restrictToHorizontalAxis } from '@dnd-kit/modifiers'
import {
  arrayMove,
  horizontalListSortingStrategy,
  SortableContext,
  sortableKeyboardCoordinates,
  useSortable,
} from '@dnd-kit/sortable'
import { CSS } from '@dnd-kit/utilities'
import { IconCrossStroked, IconPlusStroked } from '@douyinfe/semi-icons'
import { Divider, Tooltip } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import { styled } from '@linaria/react'
import React, { useEffect, useLayoutEffect, useRef, useState } from 'react'

// import StackContainer from './components/StackContainer'
const hideToNoneInterval = 3
import {
  hidden,
  flex,
  flex_1,
  items_center,
  min_w_0,
  gap_x_1,
  py_1,
  px_1,
  absolute,
  inset_0,
  visibility_hidden,
} from '@/styles/Classes'
import { ModalDialogContext } from '@/views/DialogContext'

const zIndex = css`
  z-index: 1;
`
const opacity = css`
  opacity: 0;
`

export function TabContent({
  children,
  style,
  visible,
}: {
  children?: React.ReactNode
  style?: React.CSSProperties
  visible: boolean
}) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [hiddenStyle, setHiddenStyle] = useState(!visible)
  const [visibilityStyle, setVisibilityStyle] = useState(false)
  const timer = useRef<ReturnType<typeof setTimeout>>(null)

  // 延迟设置 display: none， 因为 modal dialog 是带退出动画的，但是当退出动画没有结束就 tab 就被设置了 display: none, 就会导致退出动画停止
  // 然后导致页面重新显示的时候，modal dialog 会闪一下
  // 这里默认设置了 3 秒延迟，足够大部分场景了
  //
  useEffect(() => {
    if (visible) {
      if (timer.current) {
        clearTimeout(timer.current)
      }
      timer.current = null
      setHiddenStyle(false)
      setVisibilityStyle(false)
    } else {
      if (timer.current) {
        clearTimeout(timer.current)
      }
      timer.current = setTimeout(() => {
        setHiddenStyle(true)
        timer.current = null
      }, hideToNoneInterval * 1000)
      setVisibilityStyle(true)
    }
    return () => {
      if (timer.current) {
        clearTimeout(timer.current)
        timer.current = null
      }
    }
  }, [visible])

  return (
    <div
      ref={containerRef}
      style={style}
      className={cx(
        absolute,
        inset_0,
        visible && zIndex,
        visibilityStyle && opacity,
        hiddenStyle && hidden,
      )}
    >
      <ModalDialogContext.Provider
        value={{
          container: () => {
            if (containerRef.current === null) {
              console.warn('Container is not mounted')
            }
            return containerRef.current!
          },
        }}
      >
        {children}
      </ModalDialogContext.Provider>
    </div>
  )
}

export interface TabCardProps extends TabModel {
  active: boolean
}

export interface TabModel {
  identity: string | number
  icon?: React.ReactNode
  title: React.ReactNode
  onClose?: () => void
  onClick?: () => void
  tooltip?: string
}

export interface TabProps {
  models: TabModel[]
  onAdd?: () => void
  onReordered?: (models: TabModel[]) => void
  activeIdentity?: string | number
  className?: string
}

const backgroud = css`
  border-radius: 6px;
  &:hover {
    background-color: rgba(var(--semi-grey-2), 1);
  }
`

const clip = css`
  white-space: nowrap;
  overflow: hidden;
`

const mask = css`
  -webkit-mask-image: linear-gradient(to right, #000 0%, #000 95%, transparent 100%);
  mask-image: linear-gradient(to right, #000 0%, #000 95%, transparent 100%);
`

const maxWdith = css`
  max-width: calc(100% - 85px);
`

const closeButton = css`
  &:hover {
    color: var(--semi-color-primary-hover);
  }
  &:active {
    color: var(--semi-color-primary-active);
  }
`

const tabCardStyle = css`
  flex: 1 1 250px;
  max-width: 250px;
`

// function SortableTabCard(props: TabCardProps) {
//   const {
//     attributes,
//     listeners,
//     setNodeRef,
//     transform,
//     transition,
//     isDragging,
//   } = useSortable({ id: props.identity })

//   const style = {
//     transform: CSS.Transform.toString(transform),
//     transition,
//     zIndex: isDragging ? 101 : undefined,
//   }

//   return (
//     <div ref={setNodeRef} style={{ ...style, flex: '1 1 250px', minWidth: 0, maxWidth: '250px' }} {...attributes} {...listeners}>
//       <TabCard {...props} />
//     </div>
//   )
// }

function TabCard(props: TabCardProps) {
  const elementRef = useRef<HTMLDivElement>(null)
  const [isOverflowing, setIsOverflowing] = useState(false)

  useLayoutEffect(() => {
    const element = elementRef.current

    if (!element) {
      return
    }

    const updateFade = () => {
      setIsOverflowing(element.scrollWidth > element.clientWidth)
    }

    const resizeObserver = new ResizeObserver(updateFade)

    resizeObserver.observe(element)
    updateFade()

    return () => {
      resizeObserver.disconnect()
    }
  }, [props.title])

  const { attributes, listeners, setNodeRef, transform, transition, isDragging } = useSortable({
    id: props.identity,
  })

  const style = {
    transform: CSS.Transform.toString(transform),
    transition,
    zIndex: isDragging ? 101 : undefined,
  }

  // return (
  //   <div ref={setNodeRef} style={{ ...style, flex: '1 1 250px', minWidth: 0, maxWidth: '250px' }} {...attributes} {...listeners}>
  //     <TabCard {...props} />
  //   </div>
  // )

  const [hover, setHover] = useState(false)

  const title = (
    <div
      ref={elementRef}
      style={{ fontSize: '18px' }}
      className={cx(flex_1, flex, items_center, clip, min_w_0, isOverflowing && mask)}
    >
      {props.title}
    </div>
  )

  const component = (
    <div
      onMouseEnter={() => setHover(true)}
      onMouseLeave={() => setHover(false)}
      ref={setNodeRef}
      style={style}
      {...attributes}
      {...listeners}
      onClick={props.onClick}
      className={cx(
        flex,
        items_center,
        backgroud,
        min_w_0,
        gap_x_1,
        px_1,
        py_1,
        tabCardStyle,
        props.active &&
          css`
            background-color: var(--semi-color-fill-1);
          `,
      )}
    >
      <div className={cx(flex)}>{props.icon}</div>
      {title}
      <IconCrossStroked
        className={cx(closeButton, !hover && !props.active && hidden)}
        onClick={(e) => {
          e.stopPropagation()
          props.onClose?.()
        }}
      />
    </div>
  )

  return props.tooltip ? (
    <Tooltip showArrow={false} content={props.tooltip}>
      {component}
    </Tooltip>
  ) : (
    component
  )
}

const Icon = styled.div`
  height: 100%;
  border-radius: 4px;
  aspect-ratio: 1;
  justify-content: center;
  &:hover {
    background-color: var(--semi-color-fill-1);
  }
`

export function Tab(props: TabProps) {
  const sensors = useSensors(
    useSensor(PointerSensor, {
      activationConstraint: { distance: 5 },
    }),
    useSensor(KeyboardSensor, {
      coordinateGetter: sortableKeyboardCoordinates,
    }),
  )

  const handleDragEnd = (event: DragEndEvent) => {
    if (!props.onReordered) return
    const { active, over } = event
    if (over && active.id !== over.id) {
      const oldIndex = props.models.findIndex((item) => item.identity === active.id)
      const newIndex = props.models.findIndex((item) => item.identity === over.id)
      props.onReordered(arrayMove(props.models, oldIndex, newIndex))
    }
  }

  const divider = css`
    --semi-color-border: rgba(var(--semi-grey-9), 0.1);
  `

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
      modifiers={[restrictToHorizontalAxis]}
    >
      <SortableContext
        items={props.models.map((e) => e.identity)}
        strategy={horizontalListSortingStrategy}
      >
        <div className={cx(flex, tabContainer, maxWdith, items_center, props.className)}>
          {props.models.map((e, index) => {
            let dividerVisible = true
            if (props.models[index].identity === props.activeIdentity) {
              dividerVisible = false
            } else if (
              index < props.models.length - 1 &&
              props.models[index + 1].identity === props.activeIdentity
            ) {
              dividerVisible = false
            }
            return (
              <React.Fragment key={e.identity}>
                <TabCard
                  active={props.activeIdentity === e.identity}
                  identity={e.identity}
                  icon={e.icon}
                  title={e.title}
                  onClose={e.onClose}
                  onClick={e.onClick}
                  tooltip={e.tooltip}
                ></TabCard>
                <Divider
                  className={cx(!dividerVisible && visibility_hidden, divider)}
                  layout="vertical"
                ></Divider>
              </React.Fragment>
            )
          })}
          <Icon onClick={props.onAdd} className={cx(flex, items_center)}>
            <IconPlusStroked />
          </Icon>
        </div>
      </SortableContext>
    </DndContext>
  )
}

// 添加新的样式类（在 Tab 组件之前）
const tabContainer = css`
  overflow: hidden;
  min-width: 0px;
  flex-shrink: 1;
`
