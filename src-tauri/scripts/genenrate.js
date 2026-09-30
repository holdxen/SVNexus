import { spawnSync } from 'node:child_process'
import path from 'node:path'

import { resolveSvnPath } from './lib.js'

if (process.platform === 'win32') {
  throw new Error('Windows 暂时不支持')
}

// Node 的 process.platform → resolveSvnPath 需要的平台名
const PLATFORM_RUST = {
  win32: 'windows',
  darwin: 'darwin',
  linux: 'linux',
}

// Node 的 process.arch → Rust 风格架构名
const ARCH_RUST = {
  x64: 'x86_64',
  arm64: 'aarch64',
}

const platform = PLATFORM_RUST[process.platform]
const arch = ARCH_RUST[process.arch]

if (platform === undefined) {
  throw new Error(`Unsupported platform: ${process.platform} ${process.arch}`)
}

if (arch === undefined) {
  throw new Error(`Unsupported arch: ${process.arch}`)
}

const svnPath = resolveSvnPath(platform, arch)

const libPath = path.resolve(`src-tauri/${svnPath}/lib`)

// macOS 用 DYLD_LIBRARY_PATH，Linux 用 LD_LIBRARY_PATH，把 libPath 追加进去
const libEnvName = process.platform === 'darwin' ? 'DYLD_LIBRARY_PATH' : 'LD_LIBRARY_PATH'
const libEnvValue = process.env[libEnvName] ? `${libPath}:${process.env[libEnvName]}` : libPath

const result = spawnSync('cargo', ['test', 'export_bindings'], {
  cwd: path.resolve('src-tauri'),
  stdio: 'inherit',
  env: { ...process.env, [libEnvName]: libEnvValue },
})

if (result.error) {
  throw result.error
}

if (result.status !== 0) {
  throw new Error(`cargo test export_bindings failed with exit code ${result.status}`)
}
