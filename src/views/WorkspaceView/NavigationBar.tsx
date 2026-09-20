import { IconChevronRight } from '@douyinfe/semi-icons'
import { Breadcrumb, Dropdown } from '@douyinfe/semi-ui'
import OperationRelocateIcon from '@icons/OperationRelocate.svg?react'
import { css, cx } from '@linaria/core'
import { useState } from 'react'

import { InfoOptions } from '@/bindings/InfoOptions'
import { StatusOptions } from '@/bindings/StatusOptions'
import { WorkingCopyStatus } from '@/bindings/WorkingCopyStatus'
import OperationBar, { OperationIconProps } from '@/components/OperationBar'
import { Subversion, useSubversion } from '@/context/Subversion'
import { useModal } from '@/lib/multi-modal'
import { localPath } from '@/utils/Path'

import {
  px_2,
  py_1,
  cursor_pointer,
  cursor_default,
  flex,
  items_center,
  flex_1,
  flex_row_reverse,
  border_box,
} from '../../styles/Classes'
import { NiceRelocateDialog } from '../dialogs/RelocateDialog'
import Logger from '@/utils/Logger'

export interface NavigationBarProps {
  path: string
  root: string
  onSelected: (path: string) => void
  className?: string
}

const font = css`
  font-size: 16px;
`

const disableColor = css`
  color: var(--semi-color-text-3);
`

const enableColor = css`
  color: var(--semi-color-text-0);
`

const hover = css`
  &:hover {
    color: var(--semi-color-primary);
  }
`

const baseline = css`
  & .semi-breadcrumb-item-icon {
    vertical-align: baseline !important;
    margin-bottom: 0 !important;
  }
`
const dropdownLimit = css`
  max-height: 50vh;
  overflow-y: auto;
`

function Separator({
  enable,
  dropDown,
  onVisibleChange,
}: {
  enable: boolean
  dropDown?: React.ReactNode
  onVisibleChange?: (visible: boolean) => void
}) {
  // const color = enable ? "var(--semi-color-text-0)" : 'var(--semi-color-text-3)'
  return (
    <Dropdown
      showTick
      clickToHide={true}
      trigger={'click'}
      onVisibleChange={onVisibleChange}
      position="bottomLeft"
      render={dropDown}
      contentClassName={dropdownLimit}
    >
      <div className={cx(flex, items_center, enable && cursor_pointer, !enable && cursor_default)}>
        <IconChevronRight
          className={cx(enable && enableColor, enable && hover, !enable && disableColor)}
          size={'large'}
        ></IconChevronRight>{' '}
      </div>
    </Dropdown>
  )
}

export function NavigationBar({ path, root, onSelected, className }: NavigationBarProps) {
  const relocate = localPath.getParent(root) ?? ''
  const displayItems = localPath.intoParts(relocate)
  const routeItems = localPath.intoParts(localPath.stripPrefix(path, relocate) ?? '')

  const [folders, setFolders] = useState<
    { name: string; status: WorkingCopyStatus; active: boolean }[]
  >([])
  const [title, setTitle] = useState('')
  const [parent, setParent] = useState<string | null>(null)

  const dropDown = (
    <Dropdown.Menu>
      <Dropdown.Title key={0}>{folders.length === 0 ? `${title}(empty)` : title}</Dropdown.Title>
      {folders.map((e, index) => {
        return (
          <Dropdown.Item
            active={e.active}
            onClick={() => {
              if (parent === null) {
                return
              }

              let selected = localPath.combine([parent, e.name])

              onSelected(selected)
            }}
            key={index + 1}
          >
            {e.name}
          </Dropdown.Item>
        )
      })}
    </Dropdown.Menu>
  )

  // const workspace = useWorkspaceContext()

  const subversion = useSubversion()

  const onListFolder = async (parent: string) => {
    const options: StatusOptions = {
      path: parent,
      revision: 'working',
      depth: 'immediates',
      getAll: true,
      checkOutOfDate: false,
      checkWorkingCopy: false,
      noIgnore: false,
      ignoreExternals: true,
      changelist: null,
      depthAsSticky: false,
    }

    try {
      let context = await subversion.context()
      let result = await context.status(options)

      let items = []
      for (let i of result.entries.filter((i) => i.nodeKind == 'directory')) {
        if (i.path == parent) {
          continue
        }
        const name = localPath.stripPrefix(i.path, parent) ?? ''
        const active = localPath.startsWith(path, i.path)

        items.push({ name, status: i.nodeStatus, active })
      }
      Logger.info("items is:", items)
      setFolders(items)
    } catch (error) {}
  }

  const modal = useModal()

  const icons: OperationIconProps[] = [
    {
      sync: false,
      tooltip: 'Relocate',
      enable: true,
      children: <OperationRelocateIcon />,
      onClick: async () => {
        const options: InfoOptions = {
          path: root,
          pegRevision: 'unspecified',
          revision: 'unspecified',
          depth: 'empty',
          fetchExcluded: false,
          fetchActualOnly: false,
          includeExternals: false,
          changelists: null,
        }
        const result = await Subversion.callOnce({
          factory: subversion,
          call: async (context) => {
            return await context.info(options)
          },
        })
        if (result !== null) {
          const entries = Object.entries(result.entries)
          if (entries.length === 1) {
            const [_, value] = entries[0]
            modal.show(NiceRelocateDialog, {
              workingCopy: root,
              from: value.repositoryRootUrl ?? '',
            })
          }
        }
      },
    },
  ]

  return (
    <div className={cx(flex, items_center, className)}>
      <Breadcrumb
        showTooltip={false}
        className={cx(baseline, px_2, py_1)}
        autoCollapse={false}
        compact={false}
      >
        {displayItems.map((item) => {
          return (
            <Breadcrumb.Item
              noLink
              active={false}
              separator={<Separator enable={false}></Separator>}
            >
              <span className={cx(font, disableColor)}>{item}</span>
            </Breadcrumb.Item>
          )
        })}
        {routeItems.map((item, index) => (
          <Breadcrumb.Item
            style={{ cursor: 'pointer', display: 'flex', alignItems: 'center' }}
            active={false}
            separator={
              <Separator
                enable
                dropDown={dropDown}
                onVisibleChange={(visible) => {
                  setTitle(routeItems[index])
                  if (visible) {
                    setFolders([])
                    const current = localPath.combine([
                      ...displayItems,
                      ...routeItems.slice(0, index + 1),
                    ])
                    setParent(current)
                    onListFolder(current)
                  }
                }}
              ></Separator>
            }
            onClick={() => {
              onSelected(localPath.combine([...displayItems, ...routeItems.slice(0, index + 1)]))
            }}
          >
            <span className={cx(font, enableColor, hover)}>{item}</span>
          </Breadcrumb.Item>
        ))}
        <Breadcrumb.Item></Breadcrumb.Item>
      </Breadcrumb>
      <OperationBar
        className={cx(flex_1, border_box, px_2, flex_row_reverse)}
        icons={icons}
      ></OperationBar>
    </div>
  )
}
