const platform = process.env.TAURI_ENV_PLATFORM
const arch = process.env.TAURI_ENV_ARCH
const debug = process.env.TAURI_ENV_DEBUG === 'true'

let svn = null

if (platform === 'windows' && arch === 'x86_64') {
  svn = 'deps/win-x64/svn'
}

if (platform === 'windows' && arch === 'aarch64') {
  // Windows ARM64
  svn = 'deps/win-arm64/svn'
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

if (svn === null) {
  throw new Error(`Unsupported platform: ${platform} ${arch}`)
}

import { readdir, cp } from 'node:fs/promises'
import { rm } from 'node:fs/promises'
import path from 'node:path'

process.chdir('./src-tauri')

const dir = debug ? 'debug' : 'release'

const windowsRun = async () => {
  const srcDir = path.join(svn, 'bin')
  const destDir = `./target/${dir}`

  const entries = await readdir(srcDir, { withFileTypes: true })
  await Promise.all(
    entries.map((entry) =>
      cp(path.join(srcDir, entry.name), path.join(destDir, entry.name), { recursive: true }),
    ),
  )
}

const unixRun = async () => {
  await rm(`./target/${dir}/svnexus-svn`, {
    recursive: true,
    force: true,
  })

  await cp(svn, `./target/${dir}/svnexus-svn`, {
    recursive: true,
    dereference: false,
    verbatimSymlinks: true,
  })
}

if (platform === 'windows') {
  await windowsRun()
} else {
  await unixRun()
}
