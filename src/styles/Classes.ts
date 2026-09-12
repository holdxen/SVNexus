import { css } from '@linaria/core'

// ==================== Display ====================
export const flex = css`
  display: flex;
`

export const flex_col = css`
  flex-direction: column;
`

export const flex_1 = css`
  flex: 1 1 0%;
`

export const flex_row_reverse = css`
  flex-direction: row-reverse;
`

export const flex_5 = css`
  flex: 5 5 0%;
`

export const grid = css`
  display: grid;
`

export const hidden = css`
  display: none;
`

export const visibility_hidden = css`
  visibility: hidden;
`

export const relative = css`
  position: relative;
`

// ==================== Sizing ====================
export const w_full = css`
  width: 100%;
`

export const h_full = css`
  height: 100%;
`
export const h_auto = css`
  height: auto;
`

export const min_h_0 = css`
  min-height: 0px;
`

export const min_w_0 = css`
  min-width: 0px;
`

// ==================== Overflow ====================
export const overflow_hidden = css`
  overflow: hidden;
`

export const overflow_visible = css`
  overflow: visible;
`

export const overflow_y_auto = css`
  overflow-y: auto;
`

// ==================== Cursor ====================
export const cursor_pointer = css`
  cursor: pointer;
`

export const cursor_default = css`
  cursor: default;
`

export const cursor_grabbing = css`
  cursor: grabbing;
`

// ==================== Flexbox & Grid Alignment ====================
export const items_center = css`
  align-items: center;
`

export const items_end = css`
  align-items: end;
`

export const self_end = css`
  align-self: end;
`

export const self_center = css`
  align-self: center;
`
export const justify_center = css`
  justify-content: center;
`

// ==================== Gap ====================
export const gap_x_1 = css`
  column-gap: 0.25rem;
`

export const gap_x_2 = css`
  column-gap: 0.5rem;
`

export const gap_x_4 = css`
  column-gap: 1rem;
`

export const gap_y_1 = css`
  row-gap: 0.25rem;
`

export const gap_y_2 = css`
  row-gap: 0.5rem;
`

export const gap_y_3 = css`
  row-gap: 0.75rem;
`

// ==================== Margin ====================
export const m_1 = css`
  margin: 0.25rem;
`

export const m_2 = css`
  margin: 0.5rem;
`

export const mx_1 = css`
  margin-left: 0.25rem;
  margin-right: 0.25rem;
`

export const my_1 = css`
  margin-top: 0.25rem;
  margin-bottom: 0.25rem;
`

export const my_2 = css`
  margin-top: 0.5rem;
  margin-bottom: 0.5rem;
`

export const my_6 = css`
  margin-top: 1.5rem;
  margin-bottom: 1.5rem;
`

// ==================== Padding ====================
export const p_0 = css`
  padding-left: 0px;
  padding-right: 0px;
  padding-top: 0px;
  padding-bottom: 0px;
`
export const p_1px = css`
  padding: 1px;
`

export const pt_1 = css`
  padding-top: 0.25rem;
`

export const pb_1 = css`
  padding-bottom: 0.25rem;
`

export const pb_3 = css`
  padding-bottom: 0.75rem;
`

export const pr_1 = css`
  padding-right: 0.25rem;
`
export const pl_1 = css`
  padding-left: 0.25rem;
`
export const p_1 = css`
  padding-left: 0.25rem;
  padding-right: 0.25rem;
  padding-top: 0.25rem;
  padding-bottom: 0.25rem;
`

export const p_2 = css`
  padding-left: 0.5rem;
  padding-right: 0.5rem;
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
`

export const p_4 = css`
  padding-left: 1rem;
  padding-right: 1rem;
  padding-top: 1rem;
  padding-bottom: 1rem;
`

export const px_1 = css`
  padding-left: 0.25rem;
  padding-right: 0.25rem;
`
export const px_2 = css`
  padding-left: 0.5rem;
  padding-right: 0.5rem;
`

export const px_3 = css`
  padding-left: 0.75rem;
  padding-right: 0.75rem;
`
export const px_1px = css`
  padding-left: 1px;
  padding-right: 1px;
`

export const px_2px = css`
  padding-left: 2px;
  padding-right: 2px;
`

export const px_6px = css`
  padding-left: 6px;
  padding-right: 6px;
`

export const py_1 = css`
  padding-top: 0.25rem;
  padding-bottom: 0.25rem;
`

export const py_2 = css`
  padding-top: 0.5rem;
  padding-bottom: 0.5rem;
`

export const py_2px = css`
  padding-top: 2px;
  padding-bottom: 2px;
`

export const py_4px = css`
  padding-top: 4px;
  padding-bottom: 4px;
`
export const py_5px = css`
  padding-top: 5px;
  padding-bottom: 5px;
`

export const py_6px = css`
  padding-top: 6px;
  padding-bottom: 6px;
`

export const py_7px = css`
  padding-top: 7px;
  padding-bottom: 7px;
`

export const py_4 = css`
  padding-top: 1rem;
  padding-bottom: 1rem;
`

export const py_6 = css`
  padding-top: 1.5rem;
  padding-bottom: 1.5rem;
`
export const py_7 = css`
  padding-top: 1.75rem;
  padding-bottom: 1.75rem;
`

export const py_8 = css`
  padding-top: 2rem;
  padding-bottom: 2rem;
`

// ==================== Typography ====================
export const font_normal = css`
  font-weight: 400;
`

export const text_12px = css`
  font-size: 12px;
`

// ==================== Grid Template Columns ====================
export const grid_cols_auto_auto_auto_auto_auto_auto_1fr_auto_auto = css`
  grid-template-columns: auto auto auto auto auto auto 1fr auto auto;
`
export const grid_cols_auto_auto_auto_1fr_auto = css`
  grid-template-columns: auto auto auto 1fr auto;
`

export const grid_cols_auto_1fr_auto = css`
  grid-template-columns: auto 1fr auto;
`

export const grid_cols_1fr = css`
  grid-template-columns: 1fr;
`

export const grid_cols_1fr_1fr = css`
  grid-template-columns: 1fr 1fr;
`

export const grid_cols_1fr_auto = css`
  grid-template-columns: 1fr auto;
`

export const grid_cols_auto_minmax_0_1fr = css`
  grid-template-columns: auto minmax(0, 1fr);
`

// ==================== Grid Template Rows ====================
export const grid_rows_auto_auto_1fr = css`
  grid-template-rows: auto auto 1fr;
`

export const grid_rows_auto_auto_auto_1fr = css`
  grid-template-rows: auto auto auto 1fr;
`
export const grid_rows_auto_auto_minmax_0_1fr = css`
  grid-template-rows: auto auto minmax(0, 1fr);
`

export const grid_rows_1fr = css`
  grid-template-rows: 1fr;
`
export const grid_rows_auto_1fr = css`
  grid-template-rows: auto 1fr;
`

export const grid_rows_1fr_auto_1fr = css`
  grid-template-rows: 1fr auto 1fr;
`

export const grid_rows_minmax_0_1fr = css`
  grid-template-rows: minmax(0, 1fr);
`

export const absolute = css`
  position: absolute;
`

export const inset_0 = css`
  inset: 0;
`

export const whitespace_nowrap = css`
  white-space: nowrap;
`

export const border_box = css`
  box-sizing: border-box;
`

export const break_all = css`
  word-break: break-all;
`

export const break_word = css`
  word-wrap: break-word;
`

export const flex_shrink_0 = css`
  flex-shrink: 0;
`

export const flex_wrap = css`
  flex-wrap: wrap;
`

export const w_auto = css`
  width: auto;
`

export const bg_red = css`
  background-color: red;
`

export const border_radius_5 = css`
  border-radius: 5px;
`

export const text_center = css`
  text-align: center;
`

export const select_none = css`
  user-select: none;
`

export const select_text = css`
  user-select: text;
`
