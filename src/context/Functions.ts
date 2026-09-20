import { encode } from '@msgpack/msgpack'

import { ExtendedVersion } from '@/bindings/ExtendedVersion'
import { ExternalApplication } from '@/bindings/ExternalApplication'
import { FrontendError } from '@/bindings/FrontendError'
import { ReplyMessage } from '@/bindings/ReplyMessage'
import { SourceLocation } from '@/bindings/SourceLocation'
import { invokeMessagePack } from '@/utils/MessagePack'

export function replySuccess<T>(id: number, value: T): Promise<void> {
  const bytes = encode(value)
  const message: ReplyMessage = {
    success: bytes,
  }
  return invokeMessagePack<void>('reply', { id, value: message })
}

export function replyFailure(id: number, value: FrontendError): Promise<void> {
  const message: ReplyMessage = {
    failure: value,
  }
  return invokeMessagePack<void>('reply', { id, value: message })
}

export function reply(id: number, value: unknown): Promise<void> {
  return invokeMessagePack<void>('reply', { id, value })
}

export function fsReadLink(path: string): Promise<string> {
  return invokeMessagePack<string>('fs_read_link', { path })
}

export function formatSize(size: number): Promise<string> {
  return invokeMessagePack<string>('format_size', { size })
}

export function databaseRevisionLocation(
  repository: string,
  path: string,
  pegRevision: number,
  revision: number,
): Promise<string | null> {
  return invokeMessagePack<string | null>('database_revision_location', {
    repository,
    path,
    pegRevision,
    revision,
  })
}

export function databaseUpdateRevisionLocation(
  repository: string,
  path: string,
  pegRevision: number,
  revision: number,
  location: string,
): Promise<void> {
  return invokeMessagePack<void>('database_update_revision_location', {
    repository,
    path,
    pegRevision,
    revision,
    location,
  })
}

export function base64Decode(data: string): Promise<Uint8Array> {
  return invokeMessagePack('base64_decode', { data })
}

export function base64Encode(data: Uint8Array, breakLines: boolean): Promise<string> {
  return invokeMessagePack('base64_encode', { data, breakLines })
}

export function extendedVersion(verbose: boolean): Promise<ExtendedVersion> {
  return invokeMessagePack('extended_version', { verbose })
}

export function logInfo(message: string, location: SourceLocation): Promise<void> {
  return invokeMessagePack('log_info', { message, location })
}

export function logError(message: string, location: SourceLocation): Promise<void> {
  return invokeMessagePack('log_error', { message, location })
}

export function logWarn(message: string, location: SourceLocation): Promise<void> {
  return invokeMessagePack('log_warn', { message, location })
}

export function logDebug(message: string, location: SourceLocation): Promise<void> {
  return invokeMessagePack('log_debug', { message, location })
}

export function logTrace(message: string, location: SourceLocation): Promise<void> {
  return invokeMessagePack('log_trace', { message, location })
}

export function OpenInExternalApplication(app: ExternalApplication, path?: string): Promise<void> {
  return invokeMessagePack('open_in_external_application', { app, path })
}
