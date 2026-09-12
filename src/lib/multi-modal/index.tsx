import React, {
  createContext,
  useContext,
  useReducer,
  useCallback,
  useMemo,
  useEffect,
  ReactNode,
  ComponentType,
} from 'react'

// Types
interface ModalState {
  id: string
  component: ComponentType<any>
  visible: boolean
  delayVisible?: boolean
  mounted?: boolean
  keepMounted?: boolean
  args?: Record<string, unknown>
  resolve?: (value: unknown) => void
  reject?: (reason?: unknown) => void
  hideResolve?: (value: unknown) => void
  hideReject?: (reason?: unknown) => void
}

interface ModalStore {
  [key: string]: ModalState
}

type ModalAction =
  | {
      type: 'show'
      payload: {
        id: string
        component: ComponentType<any>
        args?: Record<string, unknown>
        resolve: (v: unknown) => void
        reject: (r?: unknown) => void
      }
    }
  | { type: 'mounted'; payload: { id: string } }
  | {
      type: 'hide'
      payload: {
        id: string
        resolve?: (v: unknown) => void
        reject?: (r?: unknown) => void
      }
    }
  | { type: 'remove'; payload: { id: string } }

let nextModalId = 0

function createModalId(): string {
  return `modal_${nextModalId++}`
}

// Reducer
function modalReducer(state: ModalStore, action: ModalAction): ModalStore {
  switch (action.type) {
    case 'show': {
      const { id, component } = action.payload
      return {
        ...state,
        [id]: {
          id,
          component,
          visible: false,
          delayVisible: true,
          mounted: false,
          args: action.payload.args,
          resolve: action.payload.resolve,
          reject: action.payload.reject,
        },
      }
    }
    case 'mounted': {
      const { id } = action.payload
      if (!state[id]) return state
      return {
        ...state,
        [id]: { ...state[id], visible: true, delayVisible: false, mounted: true },
      }
    }
    case 'hide': {
      const { id } = action.payload
      if (!state[id]) return state
      return {
        ...state,
        [id]: {
          ...state[id],
          visible: false,
          hideResolve: action.payload.resolve,
          hideReject: action.payload.reject,
        },
      }
    }
    case 'remove': {
      const { id } = action.payload
      const { [id]: _, ...rest } = state
      return rest
    }
    default:
      return state
  }
}

// Context for modal state
interface ModalContextValue {
  store: ModalStore
  dispatch: React.Dispatch<ModalAction>
}

const ModalContext = createContext<ModalContextValue | null>(null)

// Context for current modal (used in modal components)
const CurrentModalContext = createContext<string | null>(null)

// Provider - 每个 tab 独立的 provider
export function ModalProvider({ children }: { children: ReactNode }) {
  const [store, dispatch] = useReducer(modalReducer, {})
  const value = useMemo(() => ({ store, dispatch }), [store, dispatch])
  return (
    <ModalContext.Provider value={value}>
      {children}
      <ModalRenderer />
    </ModalContext.Provider>
  )
}

// 自动渲染所有 modal
function ModalRenderer() {
  const context = useContext(ModalContext)
  if (!context) return null

  const { store, dispatch } = context

  return (
    <>
      {Object.entries(store).map(([key, modal]) => {
        // remove 后不再渲染
        if (!modal.visible && !modal.delayVisible && !modal.keepMounted && !modal.mounted)
          return null
        const ModalComponent = modal.component
        return (
          <CurrentModalContext.Provider key={key} value={modal.id}>
            <ModalMountDetector modal={modal} dispatch={dispatch}>
              <ModalComponent {...(modal.args || {})} />
            </ModalMountDetector>
          </CurrentModalContext.Provider>
        )
      })}
    </>
  )
}

// 检测组件第一次挂载，触发 delayVisible → visible 转换
function ModalMountDetector({
  modal,
  dispatch,
  children,
}: {
  modal: ModalState
  dispatch: React.Dispatch<ModalAction>
  children: ReactNode
}) {
  useEffect(() => {
    if (modal.delayVisible) {
      dispatch({ type: 'mounted', payload: { id: modal.id } })
    }
  }, [dispatch, modal.delayVisible, modal.id])

  return <>{children}</>
}

// Hook - 在触发 dialog 的地方使用
export function useModal() {
  const context = useContext(ModalContext)
  if (!context) throw new Error('useModal must be used within a ModalProvider')

  const { dispatch } = context

  const show = useCallback(
    <P extends Record<string, unknown> = Record<string, unknown>>(
      component: ComponentType<P>,
      args?: P,
    ) => {
      const id = createModalId()
      const promise = new Promise<unknown>((resolve, reject) => {
        dispatch({ type: 'show', payload: { id, component, args, resolve, reject } })
      })
      return {
        id,
        as: <T extends unknown = unknown>() => promise as Promise<T>,
        promise,
        hide: () =>
          new Promise<unknown>((resolve, reject) => {
            dispatch({ type: 'hide', payload: { id, resolve, reject } })
          }),
        remove: () => dispatch({ type: 'remove', payload: { id } }),
      }
    },
    [dispatch],
  )

  const hide = useCallback(
    (id: string) => {
      return new Promise<unknown>((resolve, reject) => {
        dispatch({ type: 'hide', payload: { id, resolve, reject } })
      })
    },
    [dispatch],
  )

  const remove = useCallback(
    (id: string) => {
      dispatch({ type: 'remove', payload: { id } })
    },
    [dispatch],
  )

  return { show, hide, remove }
}

// Hook - 在 modal 组件内部使用
export function useCurrentModal() {
  const modalContext = useContext(ModalContext)
  const currentModalId = useContext(CurrentModalContext)

  if (!modalContext) throw new Error('useCurrentModal must be used within a ModalProvider')
  if (!currentModalId) throw new Error('useCurrentModal must be used inside a modal component')

  const { store, dispatch } = modalContext
  const modal = store[currentModalId]

  const hide = useCallback(() => {
    console.log('dispatch hide')
    dispatch({ type: 'hide', payload: { id: currentModalId } })
  }, [dispatch, currentModalId])

  const remove = useCallback(() => {
    dispatch({ type: 'remove', payload: { id: currentModalId } })
  }, [dispatch, currentModalId])

  const resolve = useCallback(
    (value?: unknown) => {
      modal?.resolve?.(value)
    },
    [modal],
  )

  const reject = useCallback(
    (reason?: unknown) => {
      modal?.reject?.(reason)
    },
    [modal],
  )

  const resolveHide = useCallback(
    (value?: unknown) => {
      modal?.hideResolve?.(value)
    },
    [modal],
  )

  return {
    visible: modal.visible ?? false,
    hide,
    remove,
    resolve,
    reject,
    resolveHide,
  }
}
