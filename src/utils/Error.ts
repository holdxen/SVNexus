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
    } else if ('invalidID' in error) {
      return error.invalidID.source
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
