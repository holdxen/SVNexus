const platform = process.env.TAURI_ENV_PLATFORM
const arch = process.env.TAURI_ENV_ARCH

let svn = null

if (platform === 'windows' && arch === 'x86_64') {
  svn = 'deps/win-x64/svn'
}

if (platform === 'windows' && arch === 'aarch64') {
  // Windows ARM64
  svn = 'deps/win-aarch64/svn'
}

if (platform === 'darwin' && arch === 'aarch64') {
  // macOS Apple Silicon
  svn = 'deps/macos-aarch64/svn'
}

if (platform === 'darwin' && arch === 'x86_64') {
  // macOS Intel
  svn = 'deps/macos-x64/svn'
}

if (platform === 'linux' && arch === 'x86_64') {
  // Linux x64
  svn = 'deps/linux-x64/svn'
}

if (platform === 'linux' && arch === 'aarch64') {
  // Linux aarch64
  svn = 'deps/linux-aarch64/svn'
}

if (svn === null) {
  throw new Error(`Unsupported platform: ${platform} ${arch}`)
}

import { cp } from 'node:fs/promises'
import { rm } from 'node:fs/promises'

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
