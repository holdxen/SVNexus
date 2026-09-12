import path from 'node:path'

import { parseSync } from 'oxc-parser'
import type { Plugin } from 'vite'

// 注入目标:src/utils/Logger.ts 里 default 导出的 Logger 类的静态变参方法。
// 作者写 Logger.info("a", obj),编译期在调用的最后追加一个 SourceLocation 实参 ->
//   Logger.info("a", obj, { file, member, line })
// 等价于 C++ std::source_location。Logger 的方法再把最后这个参数取出当 location,
// 其余参数拼成 message,转调 Functions.ts 的 log*(message, location)。
const LOGGER_METHODS = new Set(['info', 'error', 'warn', 'debug', 'trace'])

// Logger 模块的 import 源(@/utils/Logger、./Logger、../utils/Logger 等)去掉扩展名后以 Logger 结尾。
const LOGGER_MODULE_RE = /(?:^|\/)Logger$/

// 预过滤:文件里没出现 Logger 字样就跳过解析,避免对每个模块都做无谓的 AST parse。
const QUICK_RE = /Logger/

interface Edit {
  pos: number
  text: string
}

function walk(node: any, cb: (n: any) => void): void {
  if (!node || typeof node !== 'object') return
  if (Array.isArray(node)) {
    for (const child of node) walk(child, cb)
    return
  }
  if (typeof node.type === 'string') cb(node)
  for (const key in node) {
    if (key === 'type') continue
    const value = node[key]
    if (value && typeof value === 'object') walk(value, cb)
  }
}

function langOf(id: string): 'js' | 'jsx' | 'ts' | 'tsx' {
  if (id.endsWith('.tsx')) return 'tsx'
  if (id.endsWith('.jsx')) return 'jsx'
  if (id.endsWith('.ts')) return 'ts'
  return 'js'
}

export function injectSourceLocation(): Plugin {
  let root = process.cwd()

  return {
    name: 'svnexus:inject-source-location',
    // 必须排在 react-compiler / wyw 之前:这样拿到的是作者原始源码,
    // 注入的行号才与文件中的真实行号一致(后续 transform 会改写代码、移动行列)。
    enforce: 'pre',
    configResolved(config) {
      root = config.root
    },
    transform(code, id) {
      const cleanId = id.split('?')[0]
      if (cleanId.startsWith('\0')) return null
      if (cleanId.includes('/node_modules/')) return null
      if (!/\.(?:m|c)?[jt]sx?$/.test(cleanId)) return null
      if (!QUICK_RE.test(code)) return null

      // 实测:oxc-parser 返回的 start/end 是 JS 字符串(UTF-16)下标,可直接用于 code.slice,
      // 即便源码含中文/emoji 也不会错位;节点没有 loc,行号用偏移自己算。
      const result = parseSync(cleanId, code, { lang: langOf(cleanId) })
      if (result.errors && result.errors.length > 0) return null

      // 1) 找出本文件里 default-import 自 Logger 模块的本地绑定名。
      //    支持 `import Logger from '@/utils/Logger'` 以及任意别名 / 带扩展名写法。
      const loggerNames = new Set<string>()
      walk(result.program, (node) => {
        if (node.type !== 'ImportDeclaration') return
        const raw = node.source?.value
        if (typeof raw !== 'string') return
        const base = raw.replace(/\.(?:m|c)?[jt]sx?$/, '')
        if (!LOGGER_MODULE_RE.test(base)) return
        for (const spec of node.specifiers ?? []) {
          if (spec?.type === 'ImportDefaultSpecifier' && spec.local?.name) {
            loggerNames.add(spec.local.name)
          }
        }
      })
      if (loggerNames.size === 0) return null

      // 2) 偏移 -> 行号 索引。
      const lineStarts = [0]
      for (let i = 0; i < code.length; i++) {
        if (code[i] === '\n') lineStarts.push(i + 1)
      }
      const lineOf = (offset: number): number => {
        let lo = 0
        let hi = lineStarts.length - 1
        while (lo < hi) {
          const mid = (lo + hi + 1) >> 1
          if (lineStarts[mid] <= offset) lo = mid
          else hi = mid - 1
        }
        return lo + 1
      }

      const file = path.relative(root, cleanId).split(path.sep).join('/')

      // 3) 匹配 <LoggerBinding>.<method>(...) 调用,在右括号前把 location 追加为最后一个实参。
      const edits: Edit[] = []
      walk(result.program, (node) => {
        if (node.type !== 'CallExpression') return
        const callee = node.callee
        if (!callee || callee.type !== 'MemberExpression' || callee.computed) return
        const obj = callee.object
        const prop = callee.property
        if (!obj || obj.type !== 'Identifier' || !loggerNames.has(obj.name)) return
        if (!prop || prop.type !== 'Identifier' || !LOGGER_METHODS.has(prop.name)) return

        // node.end 指向 ')' 之后一位,右括号在 end-1。变参可有 0..N 个实参,一律追加。
        const closeParen = node.end - 1
        let p = closeParen - 1
        while (p >= 0 && /\s/.test(code[p])) p--
        // 空参 info() 前一个非空白字符是 '('、尾逗号 info(x,) 是 ',' —— 这两种都不再补逗号。
        const prev = code[p]
        const needComma = prev !== ',' && prev !== '('
        const loc = `{ file: ${JSON.stringify(file)}, member: "", line: ${lineOf(node.start)} }`
        edits.push({ pos: closeParen, text: (needComma ? ', ' : '') + loc })
      })
      if (edits.length === 0) return null

      // 从后往前插入,避免前面的偏移失效。注入内容不含换行,原始行号保持不变。
      edits.sort((a, b) => b.pos - a.pos)
      let out = code
      for (const e of edits) out = out.slice(0, e.pos) + e.text + out.slice(e.pos)

      // 不提供 sourcemap:注入是同行内插入,行号不变,仅调用行插入点之后的列号有微小偏移,
      // 对「日志里的 file:line 定位」毫无影响。
      return { code: out, map: null }
    },
  }
}
