import StatusAddedIcon from '@icons/StatusAdded.svg?react'
import StatusDeletedIcon from '@icons/StatusDeleted.svg?react'
import StatusModifiedIcon from '@icons/StatusModified.svg?react'
import StatusReplacedIcon from '@icons/StatusReplaced.svg?react'
import { forwardRef, SVGProps } from 'react'

import { LogChangedPathAction } from '@/bindings/LogChangedPathAction'

const ChangeActionIcon = forwardRef<SVGSVGElement, { action: LogChangedPathAction }>(
  (props, ref) => {
    const { action, ...rest } = props // 从 props 中解构并移除 action
    const iconProps = { ref, ...rest } as SVGProps<SVGSVGElement>
    switch (
      action // 直接使用解构出来的 action 变量
    ) {
      case 'add':
        return <StatusAddedIcon {...iconProps} />
      case 'delete':
        return <StatusDeletedIcon {...iconProps} />
      case 'modify':
        return <StatusModifiedIcon {...iconProps} />
      case 'replace':
        return <StatusReplacedIcon {...iconProps} />
      default:
        return <></>
    }
  },
)

export default ChangeActionIcon
