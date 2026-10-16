// import { AprError } from '@/bindings/AprError'
import { Error } from '@/bindings/Error'
import { SubversionError } from '@/bindings/SubversionError'

export function isError(value: any): value is Error {
  if (typeof value !== 'object' || value === null) return false

  const keys = [
    'aprError',
    'subversionError',
    'invalidArgument',
    'iOError',
    'invalidID',
    'generalError',
    'runtimeError',
    'whatever',
    'jsonError',
    'enumParseError',
    'tauriError',
    'cacheBrokenError',
    'databaseError',
    'whichError',
    'globError',
    'unexpectedMessage',
  ]

  return keys.some((key) => key in value)
}

export function getSubversionError(error: any): SubversionError | undefined {
  if (typeof error === 'object') {
    if ('subversionError' in error) {
      if (typeof error.subversionError === 'object') {
        if ('source' in error.subversionError) {
          if (typeof error.subversionError.source === 'object') {
            return error.subversionError.source as SubversionError
          }
        }
      }
    }
  }
  return undefined
}

export function errorIsClientDirectory(error: SubversionError): boolean {
  return error.code === 'clientIsDirectory'
}

/**
 * 是不是「没有读权限」。
 *
 * 用于逐层向下找可浏览的根时，判断该不该继续往下一层试：
 * - 170001 SVN_ERR_RA_NOT_AUTHORIZED：svn:// 的 Authorization failed
 * - 175013 SVN_ERR_RA_DAV_FORBIDDEN：HTTP 403（serf 把 403 映射成它，见 libsvn_ra_serf/util.c）
 *
 * 注意 170013（SVN_ERR_RA_CANNOT_CREATE_SESSION，"Unable to connect to a repository at URL"）
 * 不是权限码，只是外壳：libsvn_ra/ra_loader.c 把真正的错误作为它的 child 抛出，而
 * Rust 侧会把 child 链展开进 info[]，所以上面那两个码通常已经能扫到。
 * 绝不能把 170013 本身当权限码——它同样包装「连不上」，那样会把网络故障误判成权限问题、
 * 逐层跳过并报出误导性的「读不到任何层级」。所以外壳只按文案兜底。
 *
 * 其余错误（尤其是「路径不存在」、网络失败、认证失败）都不能当作没权限跳过，
 * 否则会把真正的错误吞掉。
 */
export function errorIsAccessDenied(error: any): boolean {
  const source = getSubversionError(error)
  if (source === undefined) {
    return false
  }

  const statuses = [source.status, ...source.info.map((info) => info.status)]
  if (statuses.includes(170001) || statuses.includes(175013)) {
    return true
  }

  // 兜底：极少数路径下 403 会被包进 175002 / 170013，那时只能看文案
  const messages = [source.msg, ...source.info.map((info) => info.msg)].map((message) =>
    message.toLowerCase(),
  )
  return messages.some(
    (message) =>
      message.includes('forbidden') ||
      message.includes('authorization failed') ||
      message.includes('not authorized'),
  )
}

// // 每个变体的处理函数，留给你填写
// function handleAprError(source: AprError): void {
//   // TODO
// }

// function handleSubversionError(source: SubversionError): void {
//   // TODO
// }

// function handleInvalidArgument(detail: string): void {
//   // TODO
// }

// function handleIOError(): void {
//   // TODO
// }

// function handleInvalidID(): void {
//   // TODO
// }

// function handleGeneralError(detail: string): void {
//   // TODO
// }

// function handleRuntimeError(): void {
//   // TODO
// }

// function handleWhatever(message: string): void {
//   // TODO
// }

// function handleJsonError(): void {
//   // TODO
// }

// function handleEnumParseError(detail: string): void {
//   // TODO
// }

// function handleTauriError(): void {
//   // TODO
// }

// function handleCacheBrokenError(uuid: string): void {
//   // TODO
// }

// function handleDatabaseError(): void {
//   // TODO
// }

// function handleWhichError(): void {
//   // TODO
// }

// function handleGlobError(): void {
//   // TODO
// }

// function handleUnexpectedMessage(detail: string): void {
//   // TODO
// }

// 主处理函数
// export function handleError(error: Error): string {
//   if ('aprError' in error) {
//     handleAprError(error.aprError.source)
//   } else if ('subversionError' in error) {
//     handleSubversionError(error.subversionError.source)
//   } else if ('invalidArgument' in error) {
//     handleInvalidArgument(error.invalidArgument.detail)
//   } else if ('iOError' in error) {
//     handleIOError()
//   } else if ('invalidID' in error) {
//     handleInvalidID()
//   } else if ('generalError' in error) {
//     handleGeneralError(error.generalError.detail)
//   } else if ('runtimeError' in error) {
//     handleRuntimeError()
//   } else if ('whatever' in error) {
//     handleWhatever(error.whatever.message)
//   } else if ('jsonError' in error) {
//     handleJsonError()
//   } else if ('enumParseError' in error) {
//     handleEnumParseError(error.enumParseError.detail)
//   } else if ('tauriError' in error) {
//     handleTauriError()
//   } else if ('cacheBrokenError' in error) {
//     handleCacheBrokenError(error.cacheBrokenError.uuid)
//   } else if ('databaseError' in error) {
//     handleDatabaseError()
//   } else if ('whichError' in error) {
//     handleWhichError()
//   } else if ('globError' in error) {
//     handleGlobError()
//   } else if ('unexpectedMessage' in error) {
//     handleUnexpectedMessage(error.unexpectedMessage.detail)
//   }
// }
//
//
export function isSubversionError(
  value: any,
): value is { subversionError: { source: SubversionError } } {
  return typeof value === 'object' && value !== null && 'subversionError' in value
}

export default function errorHumanString(error: any): string {
  if (isError(error)) {
    if ('aprError' in error) {
      return error.aprError.source.msg
    } else if ('subversionError' in error) {
      // const text = [error.subversionError.source.msg, error.subversionError.source.info.map(i => i.msg)]

      const errors = [
        { status: error.subversionError.source.status, msg: error.subversionError.source.msg },
      ]

      for (const info of error.subversionError.source.info) {
        if (errors.findIndex((i) => i.status === info.status && i.msg === info.msg) < 0) {
          errors.push({ status: info.status, msg: info.msg })
        }
      }

      return `E${error.subversionError.source.status.toString().padStart(6, '0')}: ${errors.map((i) => i.msg).join('\n')}`
    } else if ('invalidArgument' in error) {
      return error.invalidArgument.detail
    } else if ('iOError' in error) {
      return error.iOError.source
    } else if ('generalError' in error) {
      return error.generalError.detail
    } else if ('runtimeError' in error) {
      return error.runtimeError.source
    } else if ('whatever' in error) {
      return error.whatever.message
    } else if ('jsonError' in error) {
      return error.jsonError.source
    } else if ('enumParseError' in error) {
      return error.enumParseError.detail
    } else if ('tauriError' in error) {
      return ''
    } else if ('cacheBrokenError' in error) {
      return ''
    } else if ('databaseError' in error) {
      return error.databaseError.source
    } else if ('whichError' in error) {
      return error.whichError.source
    } else if ('globError' in error) {
      return error.globError.source
    } else if ('unexpectedMessage' in error) {
      return error.unexpectedMessage.detail
    } else {
      return 'Unknown error'
    }
  } else {
    return String(error)
  }
}
