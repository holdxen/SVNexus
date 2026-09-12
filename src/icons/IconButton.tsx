import { styled } from '@linaria/react'

export const IconButton = styled.div<{ size?: number; color?: string }>`
  padding: 2px;
  border-radius: 4px;
  display: flex;
  width: ${({ size }) => size || 24}px;
  height: ${({ size }) => size || 24}px;
  box-sizing: border-box;

  &:not(.disabled):not(.inactive):hover {
    background-color: var(--semi-color-fill-1);
  }

  &:not(.disabled):not(.inactive):active {
    background-color: var(--semi-color-fill-2);
  }

  &:not(.inactive) > svg {
    color: ${({ color }) => color || 'var(--semi-color-text-0)'};
  }
  &:not(.inactive).disabled > svg {
    color: ${({ color }) => color || 'var(--semi-color-disabled-text)'};
  }
`
