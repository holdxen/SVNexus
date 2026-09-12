import { Spin } from '@douyinfe/semi-ui'
import { css, cx } from '@linaria/core'
import {
  flexRender,
  getCoreRowModel,
  getFilteredRowModel,
  Row,
  useReactTable,
  type ColumnDef,
  type ColumnSizingState,
} from '@tanstack/react-table'
import { useVirtualizer } from '@tanstack/react-virtual'
import { UIEventHandler, useEffect, useRef, useState, type MouseEvent } from 'react'

import { absolute, flex, flex_1, min_h_0, relative } from '@/styles/Classes'

import { ContextMenu, ContextMenuItemModel } from './ContextMenu'

const tableStyle = css`
  color: var(--semi-color-text-0);
  background-color: var(--semi-color-bg-0);
  font-family: var(--semi-font-family-regular);
  font-size: 14px;

  thead tr {
    background-color: var(--semi-color-fill-0);
  }

  th {
    height: 48px;
    padding: 0 16px;
    color: var(--semi-color-text-0);
    font-weight: 600;
    border-bottom: 1px solid var(--semi-color-border);
    border-right: 1px solid var(--semi-color-border);
  }

  thead th {
    position: sticky;
    top: 0;
    z-index: 2;
    background-color: var(--semi-color-bg-0);
  }

  tbody tr {
    height: 48px;
    background-color: var(--semi-color-bg-0);
    transition: background-color 120ms ease;
  }

  tbody tr:hover {
    background-color: var(--semi-color-fill-0);
  }

  tbody tr[data-selected='true'],
  tbody tr[data-selected='true']:hover {
    background-color: var(--semi-color-primary-light-default);
  }

  td {
    padding: 0 16px;
    color: var(--semi-color-text-0);
    border-bottom: 1px solid var(--semi-color-border);
    border-right: 1px solid var(--semi-color-border);
  }

  td[data-selected='true'] {
    background-color: var(--semi-color-primary-light-default);
    box-shadow: inset 0 0 0 1px var(--semi-color-primary);
  }

  th:last-child,
  td:last-child {
    border-right: 0;
  }
`

export type TableSelection =
  | { type: 'cell'; rowId: string; columnId: string }
  | { type: 'row'; rowIds: string[] }

export type TableSelectionMode = 'cell' | 'row' | 'multi-row'

export interface TableProps<TData> {
  data: TData[]
  columns: ColumnDef<TData, any>[]
  className?: string
  loading?: boolean
  estimateRowHeight?: number
  overscan?: number
  // onScroll?: (element: HTMLDivElement) => void | Promise<void>
  onScroll?: UIEventHandler<HTMLDivElement>
  selectionMode?: TableSelectionMode
  onSelectionChange?: (selection: TableSelection | null) => void
  onGetRowId?: (originalRow: TData, index: number, parent?: Row<TData>) => string
  // TableProps 新增
  onTopReached?: () => void
  onBottomReached?: () => void
  threshold?: number // 距首尾多少行时触发，默认 5
  rowContextMenu?: (row: Row<TData>) => ContextMenuItemModel[] | undefined
  globalFilter?: string
}

export function Table<TData>({
  data,
  columns,
  className,
  loading = false,
  estimateRowHeight = 48,
  overscan = 10,
  onScroll,
  selectionMode,
  onSelectionChange,
  onGetRowId,
  onBottomReached,
  onTopReached,
  threshold,
  rowContextMenu,
  globalFilter,
}: TableProps<TData>) {
  'use no memo'
  const [columnSizing, setColumnSizing] = useState<ColumnSizingState>({})
  const [selection, setSelection] = useState<TableSelection | null>(null)
  const scrollContainerRef = useRef<HTMLDivElement>(null)

  const updateSelection = (nextSelection: TableSelection | null) => {
    setSelection(nextSelection)
    onSelectionChange?.(nextSelection)
  }

  const handleRowClick = (rowId: string, event: MouseEvent<HTMLTableRowElement>) => {
    if (selectionMode === 'row') {
      updateSelection({ type: 'row', rowIds: [rowId] })
      return
    }

    if (selectionMode !== 'multi-row') return

    const shouldToggle = event.ctrlKey || event.metaKey
    const currentRowIds = selection?.type === 'row' ? selection.rowIds : []
    const nextRowIds = shouldToggle
      ? currentRowIds.includes(rowId)
        ? currentRowIds.filter((id) => id !== rowId)
        : [...currentRowIds, rowId]
      : [rowId]

    updateSelection(nextRowIds.length > 0 ? { type: 'row', rowIds: nextRowIds } : null)
  }

  const handleCellClick = (rowId: string, columnId: string) => {
    if (selectionMode !== 'cell') return
    updateSelection({ type: 'cell', rowId, columnId })
  }

  const table = useReactTable({
    data,
    columns,
    getCoreRowModel: getCoreRowModel(),
    columnResizeMode: 'onChange',
    enableColumnResizing: true,
    state: { columnSizing, globalFilter },
    onColumnSizingChange: setColumnSizing,
    getRowId: onGetRowId,
    getFilteredRowModel: getFilteredRowModel(),
  })
  const rows = table.getRowModel().rows
  const rowVirtualizer = useVirtualizer<HTMLDivElement, HTMLTableRowElement>({
    count: rows.length,
    getScrollElement: () => scrollContainerRef.current,
    estimateSize: () => estimateRowHeight,
    overscan,
  })
  const virtualRows = rowVirtualizer.getVirtualItems()

  useEffect(() => {
    if (virtualRows.length === 0) return
    const firstIndex = virtualRows[0].index
    const lastIndex = virtualRows[virtualRows.length - 1].index
    const t = threshold ?? 5

    if (firstIndex <= t) {
      onTopReached?.()
    }
    if (lastIndex >= rows.length - 1 - t) {
      onBottomReached?.()
    }
  }, [virtualRows])

  // const handleScroll = (event: UIEvent<HTMLDivElement>) => {
  //   void onScroll?.(event.currentTarget)
  // }

  return (
    <div className={cx(flex, relative, className)}>
      <div
        ref={scrollContainerRef}
        className={cx(flex_1, min_h_0)}
        onScroll={onScroll}
        style={{ overflow: 'auto', scrollbarGutter: 'stable' }}
      >
        <table
          className={tableStyle}
          style={{
            width: table.getTotalSize(),
            minWidth: table.getTotalSize(),
            borderCollapse: 'separate',
            borderSpacing: 0,
            tableLayout: 'fixed',
          }}
        >
          <colgroup>
            {table.getVisibleLeafColumns().map((column) => (
              <col key={column.id} style={{ width: column.getSize() }} />
            ))}
          </colgroup>
          <thead>
            {table.getHeaderGroups().map((headerGroup) => (
              <tr key={headerGroup.id}>
                {headerGroup.headers.map((header, index) => (
                  <th
                    key={header.id}
                    style={{
                      overflow: 'visible',
                      position: 'sticky',
                      top: 0,
                      zIndex: headerGroup.headers.length - index,
                      backgroundColor: 'var(--semi-color-bg-0)',
                      boxShadow: '0 1px 0 var(--semi-color-border)',
                      textAlign: 'left',
                      verticalAlign: 'middle',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {header.isPlaceholder ? null : (
                      <div style={{ overflow: 'hidden' }}>
                        {flexRender(header.column.columnDef.header, header.getContext())}
                      </div>
                    )}
                    {header.column.getCanResize() ? (
                      <div
                        onDoubleClick={() => header.column.resetSize()}
                        onMouseDown={header.getResizeHandler()}
                        onTouchStart={header.getResizeHandler()}
                        style={{
                          position: 'absolute',
                          top: 0,
                          right: -3,
                          width: 6,
                          height: '100%',
                          cursor: 'col-resize',
                          userSelect: 'none',
                          touchAction: 'none',
                          background: header.column.getIsResizing() ? '#1677ff' : 'transparent',
                          zIndex: headerGroup.headers.length + 1,
                        }}
                      />
                    ) : null}
                  </th>
                ))}
              </tr>
            ))}
          </thead>
          <tbody>
            {virtualRows.length > 0 ? (
              <tr data-virtual-spacer="top" style={{ height: virtualRows[0].start }}>
                <td
                  colSpan={table.getVisibleLeafColumns().length}
                  style={{ height: virtualRows[0].start, padding: 0, border: 0 }}
                />
              </tr>
            ) : null}
            {virtualRows.map((virtualRow) => {
              const row = rows[virtualRow.index]
              const isRowSelected = selection?.type === 'row' && selection.rowIds.includes(row.id)

              const tr = (
                <tr
                  key={row.id}
                  data-selected={isRowSelected || undefined}
                  onClick={
                    selectionMode === 'row' || selectionMode === 'multi-row'
                      ? (event) => handleRowClick(row.id, event)
                      : undefined
                  }
                  onContextMenu={
                    selectionMode === 'row' || selectionMode === 'multi-row'
                      ? (event) => handleRowClick(row.id, event)
                      : undefined
                  }
                >
                  {row.getVisibleCells().map((cell) => {
                    const isCellSelected =
                      selection?.type === 'cell' &&
                      selection.rowId === row.id &&
                      selection.columnId === cell.column.id

                    return (
                      <td
                        key={cell.id}
                        data-selected={isCellSelected || undefined}
                        onClick={() => handleCellClick(row.id, cell.column.id)}
                        style={{
                          whiteSpace: 'nowrap',
                          overflow: 'hidden',
                          textOverflow: 'clip',
                        }}
                        title={cell.getValue() == null ? undefined : String(cell.getValue())}
                      >
                        {flexRender(cell.column.columnDef.cell, cell.getContext()) ??
                          String(cell.getValue() ?? '')}
                      </td>
                    )
                  })}
                </tr>
              )

              if (rowContextMenu) {
                return (
                  <ContextMenu key={row.id} asChild menu={rowContextMenu(row)}>
                    {tr}
                  </ContextMenu>
                )
              }

              return tr
            })}
            {rowVirtualizer.getTotalSize() > 0 ? (
              <tr
                data-virtual-spacer="bottom"
                style={{
                  height:
                    rowVirtualizer.getTotalSize() - (virtualRows[virtualRows.length - 1]?.end ?? 0),
                }}
              >
                <td
                  colSpan={table.getVisibleLeafColumns().length}
                  style={{
                    height:
                      rowVirtualizer.getTotalSize() -
                      (virtualRows[virtualRows.length - 1]?.end ?? 0),
                    padding: 0,
                    border: 0,
                  }}
                />
              </tr>
            ) : null}
          </tbody>
        </table>
        {loading ? (
          <Spin
            spinning
            wrapperClassName={cx(absolute)}
            style={{ bottom: 10, right: 10, zIndex: 10 }}
          ></Spin>
        ) : null}
      </div>
    </div>
  )
}
