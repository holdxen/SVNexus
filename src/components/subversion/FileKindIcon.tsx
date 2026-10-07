import { SVGProps } from 'react'

import { NodeKind } from '@/bindings/NodeKind'

import FileDirectoryIcon from '../../icons/FileDirectory.svg?react'
import FileNormalIcon from '../../icons/FileNormal.svg?react'
import FileSymlinkIcon from '../../icons/FileSymlink.svg?react'
import FileUnknownIcon from '../../icons/FileUnknown.svg?react'

export interface FileKindIconProps extends SVGProps<SVGSVGElement> {
  title?: string
  titleId?: string
  desc?: string
  descId?: string
  kind: NodeKind
}

export default function FileKindIcon(props: FileKindIconProps) {
  const { kind, ...others } = props
  switch (kind) {
    case 'none':
    case 'unknown':
      return <FileUnknownIcon {...others} />
    case 'directory':
      return <FileDirectoryIcon {...others} />
    case 'file':
      return <FileNormalIcon {...others} />
    case 'symlink':
      return <FileSymlinkIcon {...others} />
    default:
      return <></>
  }
}
