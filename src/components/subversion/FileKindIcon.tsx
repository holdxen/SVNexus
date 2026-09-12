import { NodeKind } from '@/bindings/NodeKind'

import FileDirectoryIcon from '../../icons/FileDirectory.svg?react'
import FileNormalIcon from '../../icons/FileNormal.svg?react'
import FileSymlinkIcon from '../../icons/FileSymlink.svg?react'
import FileUnknownIcon from '../../icons/FileUnknown.svg?react'

export default function FileKindIcon({ kind }: { kind: NodeKind }) {
  switch (kind) {
    case 'none':
    case 'unknown':
      return <FileUnknownIcon />
    case 'directory':
      return <FileDirectoryIcon />
    case 'file':
      return <FileNormalIcon />
    case 'symlink':
      return <FileSymlinkIcon />
    default:
      return <></>
  }
}
