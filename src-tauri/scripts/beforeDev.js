import { resolveSvnPath } from './lib.js'

const platform = process.env.TAURI_ENV_PLATFORM
const arch = process.env.TAURI_ENV_ARCH
const debug = process.env.TAURI_ENV_DEBUG === 'true'

const svn = resolveSvnPath(platform, arch)

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

  // 同时创建 ./target/svnexus-svn/ 目录以满足 tauri.windows.conf.json 的资源路径校验
  await rm('./target/svnexus-svn', { recursive: true, force: true })
  await cp(`${svn}/bin`, './target/svnexus-svn', { recursive: true })
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
