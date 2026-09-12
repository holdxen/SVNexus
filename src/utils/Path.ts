export default function simplifyPath(relativePath: string, path: string) {
  if (relativePath === '' || relativePath === '/') {
    return path
  }
  const relativeParts = relativePath.split('/')
  const parts = path.split('/')

  const results: string[] = []

  const len = Math.min(relativeParts.length, parts.length)

  let i = 0
  for (; i < len; i++) {
    if (relativeParts[i] !== parts[i]) {
      break
    }
  }

  results.push(...Array(relativeParts.length - i).fill('..'))

  for (let j = i; j < parts.length; j++) {
    results.push(parts[j])
  }

  return results.join('/')
}

export const isWindows = typeof navigator !== 'undefined' && navigator.userAgent.includes('Windows')

// 语义对齐 std::path::Path（后端 camino 的底层实现），
// 用于在前端替代 path_* 系列 IPC 命令
export class PathUtil {
  constructor(
    // 切分时识别的分隔符集合，如 '/' 或 '/\\'
    readonly separators: string,
    // 拼接输出使用的分隔符
    readonly mainSeparator: string,
  ) {}

  private isSeparator(ch: string): boolean {
    return this.separators.includes(ch)
  }

  // Windows 前缀长度：盘符 "C:" 或 UNC "\\server\share"；非反斜杠域恒为 0
  private prefixLength(path: string): number {
    if (!this.separators.includes('\\')) {
      return 0
    }
    if (path.length >= 2 && this.isSeparator(path[0]) && this.isSeparator(path[1])) {
      let i = 2
      while (i < path.length && !this.isSeparator(path[i])) i++
      if (i < path.length) {
        i++
        while (i < path.length && !this.isSeparator(path[i])) i++
      }
      return i
    }
    if (/^[A-Za-z]:/.test(path)) {
      return 2
    }
    return 0
  }

  // 去掉尾部分隔符和尾部 '.' 组件后的有效长度
  private trimEnd(path: string): number {
    let end = path.length
    while (end > 0) {
      while (end > 0 && this.isSeparator(path[end - 1])) end--
      let start = end
      while (start > 0 && !this.isSeparator(path[start - 1])) start--
      if (end > start && path.slice(start, end) === '.') {
        end = start
        continue
      }
      break
    }
    return end
  }

  getFileName(path: string): string | null {
    if (path === '') {
      return null
    }
    const end = this.trimEnd(path)
    if (end === 0) {
      return null
    }
    let start = end
    while (start > 0 && !this.isSeparator(path[start - 1])) start--
    const name = path.slice(start, end)
    return name === '..' ? null : name
  }

  // 扩展名不含 '.'：'a/b.png' → 'png'，'a/b.tar.gz' → 'gz'
  // 与 std::path::Path::extension 一致：'..'、'.gitignore' 视为无扩展名，'foo.' 的扩展名为空串
  getExtension(path: string): string | null {
    const name = this.getFileName(path)
    if (name === null) {
      return null
    }
    const dotIndex = name.lastIndexOf('.')
    if (dotIndex <= 0) {
      return null
    }
    return name.slice(dotIndex + 1)
  }

  getParent(path: string): string | null {
    if (path === '') {
      return null
    }
    const end = this.trimEnd(path)
    if (end === 0) {
      return null
    }

    let compStart = end
    while (compStart > 0 && !this.isSeparator(path[compStart - 1])) compStart--

    const prefixLen = this.prefixLength(path)

    if (compStart === 0) {
      // 单个组件：前缀本身（"C:"）没有父路径，普通相对路径父路径为空串
      return prefixLen > 0 ? null : ''
    }

    const sepIndex = compStart - 1
    if (sepIndex < prefixLen) {
      // 最后一个组件属于前缀的一部分（UNC 的 share）
      return null
    }

    // 根目录属于父路径：'/foo' → '/'，'C:\foo' → 'C:\'
    const rootEnd = prefixLen + (this.isSeparator(path[prefixLen] ?? '') ? 1 : 0)
    let parentEnd = sepIndex
    if (parentEnd < rootEnd) {
      parentEnd = Math.min(rootEnd, end)
    }
    return path.slice(0, parentEnd)
  }

  intoParts(path: string): string[] {
    const parts: string[] = []
    if (path === '') {
      return parts
    }

    let i = 0
    const prefixLen = this.prefixLength(path)
    if (prefixLen > 0) {
      parts.push(path.slice(0, prefixLen))
      i = prefixLen
    }

    if (i < path.length && this.isSeparator(path[i])) {
      parts.push(path[i])
      while (i < path.length && this.isSeparator(path[i])) i++
    }

    while (i < path.length) {
      let j = i
      while (j < path.length && !this.isSeparator(path[j])) j++
      const comp = path.slice(i, j)
      if (comp !== '.' && comp !== '') {
        parts.push(comp)
      }
      i = j
      while (i < path.length && this.isSeparator(path[i])) i++
    }

    if (parts.length === 0) {
      for (const ch of path) {
        if (!this.isSeparator(ch)) {
          parts.push('.')
          break
        }
      }
    }
    return parts
  }

  stripPrefix(path: string, prefix: string): string | null {
    const pathParts = this.intoParts(path)
    const prefixParts = this.intoParts(prefix)
    if (prefixParts.length > pathParts.length) {
      return null
    }
    for (let i = 0; i < prefixParts.length; i++) {
      if (pathParts[i] !== prefixParts[i]) {
        return null
      }
    }
    return this.combine(pathParts.slice(prefixParts.length))
  }

  startsWith(path: string, prefix: string): boolean {
    const pathParts = this.intoParts(path)
    const prefixParts = this.intoParts(prefix)
    if (prefixParts.length > pathParts.length) {
      return false
    }
    return prefixParts.every((part, index) => pathParts[index] === part)
  }

  combine(parts: string[], separator?: string): string {
    const sep = separator ?? this.mainSeparator
    let result = ''
    for (const part of parts) {
      if (part === '') {
        continue
      }
      if (result === '') {
        result = part
        continue
      }
      const partPrefixLen = this.prefixLength(part)
      const partIsAbsolute = partPrefixLen < part.length && this.isSeparator(part[partPrefixLen])
      if (partIsAbsolute) {
        // 无前缀的绝对路径保留已有盘符：'\' 拼到 'C:' 上得到 'C:\'
        const resultPrefixLen = this.prefixLength(result)
        if (partPrefixLen === 0 && resultPrefixLen > 0) {
          result = result.slice(0, resultPrefixLen) + part
        } else {
          result = part
        }
        continue
      }
      if (this.isSeparator(result[result.length - 1])) {
        result += part
      } else {
        result += sep + part
      }
    }
    return result
  }
}

// 仓库路径：SVN 协议保证恒为 '/'
export const repoPath = new PathUtil('/', '/')

// 本地路径：与 std::path 一致，Windows 上识别 '/' 和 '\'，输出 '\'
export const localPath = new PathUtil(isWindows ? '/\\' : '/', isWindows ? '\\' : '/')
