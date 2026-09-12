export interface OperationState {
  refresh: boolean
  add: boolean
  revert: boolean
  lock: boolean
  unlock: boolean
  diff: boolean
  patch: boolean
  delete: boolean
  info: boolean
  commit: boolean
  import: boolean
  switch: boolean
  merge: boolean
  mkdir: boolean
  update: boolean
  relocate: boolean
  copy: boolean
  move: boolean
  export: boolean
  ignore: boolean
}

export const defaultOperationState: OperationState = {
  refresh: false,
  add: false,
  revert: false,
  lock: false,
  unlock: false,
  diff: false,
  patch: false,
  delete: false,
  info: false,
  commit: false,
  import: false,
  switch: false,
  merge: false,
  mkdir: false,
  update: false,
  relocate: false,
  copy: false,
  move: false,
  export: false,
  ignore: false,
}
