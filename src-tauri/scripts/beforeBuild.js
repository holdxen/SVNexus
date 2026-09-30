import { resolveSvnPath } from './lib.js'

const platform = process.env.TAURI_ENV_PLATFORM
const arch = process.env.TAURI_ENV_ARCH

const svn = resolveSvnPath(platform, arch)

import { cp } from 'node:fs/promises'
import { rm } from 'node:fs/promises'

process.chdir('./src-tauri')

const unixRun = async () => {
  await rm('./target/svnexus-svn', {
    recursive: true,
    force: true,
  })

  await cp(svn, './target/svnexus-svn', {
    recursive: true,
    dereference: false,
    verbatimSymlinks: true,
  })

  await rm('./target/svnexus-svn/include', {
    recursive: true,
    force: true,
  })

  await rm('./target/svnexus-svn/lib/cmake', {
    recursive: true,
    force: true,
  })

  await rm('./target/svnexus-svn/lib/pkgconfig', {
    recursive: true,
    force: true,
  })
}

const windowsRun = async () => {
  await cp(`${svn}/bin`, './target/svnexus-svn', {
    recursive: true,
    dereference: false,
    verbatimSymlinks: true,
  })
}

if (platform === 'windows') {
  await windowsRun()
} else if (platform === 'darwin' || platform === 'linux') {
  await unixRun()
} else {
  throw new Error(`Unsupported platform: ${platform} ${arch}`)
}
