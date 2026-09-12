const platform = process.env.TAURI_ENV_PLATFORM
const arch = process.env.TAURI_ENV_ARCH
const debug = process.env.TAURI_ENV_DEBUG === "true";

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

import { cp } from 'node:fs/promises'
import { rm } from 'node:fs/promises'


process.chdir('./src-tauri')

const dir = debug ? 'debug' : 'release'

await rm(`./target/${dir}/svnexus-svn`, {
  recursive: true,
  force: true,
})

await cp(svn, `./target/${dir}/svnexus-svn`, {
  recursive: true,
  dereference: false,
  verbatimSymlinks: true,
})