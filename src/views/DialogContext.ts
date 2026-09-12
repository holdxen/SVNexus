import { createContext, useContext } from 'react'

export interface ModalDialog {
  container: () => HTMLElement
}

export const ModalDialogContext = createContext<ModalDialog | null>(null)

export function useModalDialog(): ModalDialog {
  const dialog = useContext(ModalDialogContext)

  if (dialog === null) {
    throw new Error('No dialog provided')
  }

  return dialog
}
