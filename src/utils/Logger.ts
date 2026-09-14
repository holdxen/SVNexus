import type { SourceLocation } from '@/bindings/SourceLocation'
import { logDebug, logError, logInfo, logTrace, logWarn } from '@/context/Functions'

// 约定:编译期插件 vite-plugins/inject-source-location.ts 会在每个 Logger.xxx(...) 调用的
// 最后追加一个 SourceLocation 实参(类似 C++ std::source_location)。所以这里把「最后一个参数」
// 取出当作 location,其余参数才是真正要打印的内容。
//
// 注意:这套依赖编译期注入,因此 Logger 只在经过 Vite(dev / build)的代码里使用才能保证 location
// 正确;若在未注入的环境(如脱离 Vite 的脚本)直接调用,最后一个业务参数会被误当成 location。
function splitLocation(data: any[]): [unknown[], SourceLocation] {
  const location = data.pop() as SourceLocation
  return [data, location]
}

// 把变参格式化成单条字符串(类似 console.log 的多参拼接,以空格分隔)。
function formatMessage(data: unknown[]): string {
  return data.map(stringify).join(' ')
}

function stringify(value: unknown): string {
  if (typeof value === 'string') return value
  if (value instanceof Error) return value.stack ?? `${value.name}: ${value.message}`
  if (typeof value === 'bigint') return `${value}n`
  try {
    return JSON.stringify(value) ?? String(value)
  } catch {
    // 循环引用等无法序列化的对象
    return String(value)
  }
}

export default class Logger {
  static info(...data: any[]): Promise<void> {
    console.log(...data)
    const [args, location] = splitLocation(data)
    return logInfo(formatMessage(args), location)
  }

  static error(...data: any[]): Promise<void> {
    console.error(...data)
    const [args, location] = splitLocation(data)
    return logError(formatMessage(args), location)
  }

  static warn(...data: any[]): Promise<void> {
    console.warn(...data)
    const [args, location] = splitLocation(data)
    return logWarn(formatMessage(args), location)
  }

  static debug(...data: any[]): Promise<void> {
    console.debug(...data)
    const [args, location] = splitLocation(data)
    return logDebug(formatMessage(args), location)
  }

  static trace(...data: any[]): Promise<void> {
    console.trace(...data)
    const [args, location] = splitLocation(data)
    return logTrace(formatMessage(args), location)
  }
}
