import { css } from '@linaria/core'

export const stack = css`
  > * {
    box-sizing: border-box;
    width: 100%;
    height: 100%;
  }
`

export const list_item = css`
  border-radius: var(--semi-border-radius-medium, 6px);
  background-color: var(---svnexus-list-item-default-background);
  transition: background-color 0.2s ease;

  &:hover {
    background-color: var(---svnexus-list-item-hover-background);
  }

  &:active {
    background-color: var(---svnexus-list-item-pressed-background);
  }

  &:disabled,
  &[aria-disabled='true'] {
    background-color: var(---svnexus-list-item-disabled-background);
    opacity: 0.65;
    cursor: not-allowed;
  }
`

export const list_item_selected = css`
  background-color: var(---svnexus-list-item-selected-background);

  &:hover {
    background-color: var(---svnexus-list-item-selected-hover-background);
  }

  &.${list_item}:disabled,
  &.${list_item}[aria-disabled='true'] {
    background-color: var(---svnexus-list-item-disabled-selected-background);
  }
`

export const disable_move = css`
  cursor: default;
`

export const context_menu_content = css`
  position: relative;
  z-index: 1050;
  min-width: 150px;
  max-width: 380px;
  padding: 4px 0;
  overflow-y: auto;
  border-radius: var(--semi-border-radius-medium, 6px);
  background: var(--semi-color-bg-3);
  box-shadow: var(--semi-shadow-elevated);
`

export const context_menu_item = css`
  display: flex;
  align-items: center;
  box-sizing: border-box;
  width: 100%;
  max-width: 380px;
  padding: 8px 16px;
  border: 0;
  border-radius: 0;
  outline: 0;
  background: transparent;
  color: var(--semi-color-text-0);
  font-family: var(--semi-font-family-regular);
  font-size: 14px;
  font-weight: 400;
  line-height: 20px;
  cursor: default;
  user-select: none;
  transition: background-color 0ms ease-out;

  &:hover,
  &[data-highlighted],
  &:focus-visible {
    background-color: var(--semi-color-fill-0);
    cursor: pointer;
  }

  &:active {
    background-color: var(--semi-color-fill-1);
  }

  &[data-disabled] {
    color: var(--semi-color-disabled-text);
    cursor: not-allowed;
  }

  &[data-disabled]:hover,
  &[data-disabled]:active,
  &[data-disabled][data-highlighted] {
    background-color: transparent;
    cursor: not-allowed;
  }
`

export const context_menu_item_danger = css`
  color: var(--semi-color-danger);
`

export const context_menu_separator = css`
  display: block;
  width: 100%;
  min-width: 100%;
  height: 1px;
  margin: 4px 0;
  clear: both;
  background-color: var(--semi-color-border);
`

export const box_shadow = css`
  box-shadow: var(--semi-shadow-elevated);
`
