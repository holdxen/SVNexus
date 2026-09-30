const SVN_DEPS = {
  'windows:x86_64': 'deps/win-x64/svn',
  'windows:aarch64': 'deps/win-aarch64/svn',
  'darwin:aarch64': 'deps/macos-aarch64/svn',
  'darwin:x86_64': 'deps/macos-x64/svn',
  'linux:x86_64': 'deps/linux-x64/svn',
  'linux:aarch64': 'deps/linux-aarch64/svn',
}

export const resolveSvnPath = (platform, arch) => {
  const svn = SVN_DEPS[`${platform}:${arch}`]

  if (svn === undefined) {
    throw new Error(`Unsupported platform: ${platform} ${arch}`)
  }

  return svn
}
