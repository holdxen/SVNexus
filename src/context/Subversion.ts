import { Toast } from '@douyinfe/semi-ui'
import { createContext, useContext } from 'react'

import { AddOptions } from '@/bindings/AddOptions'
import { BlameOptions } from '@/bindings/BlameOptions'
import { BlameResult } from '@/bindings/BlameResult'
import { CatOptions } from '@/bindings/CatOptions'
import { CatResult } from '@/bindings/CatResult'
import { CheckoutOptions } from '@/bindings/CheckoutOptions'
import { CleanupOptions } from '@/bindings/CleanupOptions'
import { ClientDifferenceOptions } from '@/bindings/ClientDifferenceOptions'
import { ClientDifferenceResult } from '@/bindings/ClientDifferenceResult'
import { CommitOptions } from '@/bindings/CommitOptions'
import { CommitResult } from '@/bindings/CommitResult'
import { ConflictWalkOptions } from '@/bindings/ConflictWalkOptions'
import { ConflictWalkResult } from '@/bindings/ConflictWalkResult'
import { CopyOptions } from '@/bindings/CopyOptions'
import { CopyResult } from '@/bindings/CopyResult'
import { DeleteOptions } from '@/bindings/DeleteOptions'
import { DeleteResult } from '@/bindings/DeleteResult'
import { ExportOptions } from '@/bindings/ExportOptions'
import { ImportOptions } from '@/bindings/ImportOptions'
import { ImportResult } from '@/bindings/ImportResult'
import { InfoOptions } from '@/bindings/InfoOptions'
import { InfoResult } from '@/bindings/InfoResult'
import { ListOptions } from '@/bindings/ListOptions'
import { ListResult } from '@/bindings/ListResult'
import { LockOptions } from '@/bindings/LockOptions'
import { LogEntry } from '@/bindings/LogEntry'
import { LogOptions } from '@/bindings/LogOptions'
import { LogResult } from '@/bindings/LogResult'
import { MergeOptions } from '@/bindings/MergeOptions'
import { MkdirOptions } from '@/bindings/MkdirOptions'
import { MkdirResult } from '@/bindings/MkdirResult'
import { MoveOptions } from '@/bindings/MoveOptions'
import { MoveResult } from '@/bindings/MoveResult'
import { PatchOptions } from '@/bindings/PatchOptions'
import { PropertyGetOptions } from '@/bindings/PropertyGetOptions'
import { PropertyGetResult } from '@/bindings/PropertyGetResult'
import { PropertyListOptions } from '@/bindings/PropertyListOptions'
import { PropertyListResult } from '@/bindings/PropertyListResult'
import { PropertySetOptions } from '@/bindings/PropertySetOptions'
import { RelocateOptions } from '@/bindings/RelocateOptions'
import { RevertOptions } from '@/bindings/RevertOptions'
import { Revision } from '@/bindings/Revision'
import { RevisionPropertyListOptions } from '@/bindings/RevisionPropertyListOptions'
import { RevisionPropertyListResult } from '@/bindings/RevisionPropertyListResult'
import { StatusOptions } from '@/bindings/StatusOptions'
import { StatusResult } from '@/bindings/StatusResult'
import { SubversionEvent } from '@/bindings/SubversionEvent'
import { SwitchOptions } from '@/bindings/SwitchOptions'
import { UnlockOptions } from '@/bindings/UnlockOptions'
import { UpdateOptions } from '@/bindings/UpdateOptions'
import { UpgradeOptions } from '@/bindings/UpgradeOptions'
import { UpgradeResult } from '@/bindings/UpgradeResult'
import { VacuumOptions } from '@/bindings/VacuumOptions'
import { Version } from '@/bindings/Version'
import { WcReplacedNode } from '@/bindings/WcReplacedNode'
import { WorkingCopyRevisionStatusOptions } from '@/bindings/WorkingCopyRevisionStatusOptions'
import { WorkingCopyRevisionStatusResult } from '@/bindings/WorkingCopyRevisionStatusResult'
import errorHumanString from '@/utils/Error'
import { MessagePackChannel, invokeMessagePack } from '@/utils/MessagePack'

export type SubversionEventMap = {
  [K in SubversionEvent as keyof K]: K[keyof K]
}

// type EventName = keyof EventMap
// type EventListener<T> = (data: T) => void

export interface SubversionFactory {
  context: () => Promise<Subversion>
  release: (subversion: Subversion) => void
  releaseAll: () => void
}
export class Subversion {
  // private count: number = 0
  private _id: number
  private listeners: Partial<Record<keyof SubversionEventMap, Set<(data: any) => void>>> = {}
  // private doCheck: boolean = false

  get id(): number {
    return this._id
  }

  static async create(): Promise<Subversion> {
    const channel = new MessagePackChannel<SubversionEvent>()
    const id = await invokeMessagePack<number>('subversion_create', {
      channel: channel,
      config: {},
    })
    return new Subversion(id, channel)
  }

  constructor(id: number, channel: MessagePackChannel<SubversionEvent>) {
    this._id = id
    // this.doCheck = doCheck

    channel.onmessage = this.onEvent.bind(this)
  }

  on<K extends keyof SubversionEventMap>(
    event: K,
    listener: (data: SubversionEventMap[K]) => void,
  ): void {
    if (!this.listeners[event]) {
      this.listeners[event] = new Set()
    }
    this.listeners[event]!.add(listener as (data: any) => void)
  }

  off<K extends keyof SubversionEventMap>(
    event: K,
    listener: (data: SubversionEventMap[K]) => void,
  ): void {
    this.listeners[event]?.delete(listener as (data: any) => void)
  }

  private onEvent(event: SubversionEvent) {
    // const bytes = data instanceof Uint8Array ? data : new Uint8Array(data)
    for (const key of Object.keys(event)) {
      const name = key as keyof SubversionEventMap
      const data = (event as any)[name]
      const listeners = this.listeners[name]
      if (listeners) {
        for (const listener of listeners) {
          listener(data)
          return
        }
      }
    }
    console.warn('No match event handler: ', event)
  }

  public cancel(msg: string): Promise<void> {
    return invokeMessagePack('subversion_cancel', { id: this.id, msg })
  }
  public status(options: StatusOptions): Promise<StatusResult> {
    return invokeMessagePack('subversion_status', { id: this.id, options: options })
  }

  // --- Status & Info ---

  public info(options: InfoOptions): Promise<InfoResult> {
    return invokeMessagePack('subversion_info', { id: this.id, options: options })
  }

  public list(options: ListOptions): Promise<ListResult> {
    return invokeMessagePack('subversion_list', { id: this.id, options: options })
  }

  public log(options: LogOptions): Promise<LogResult> {
    return invokeMessagePack('subversion_log', { id: this.id, options: options })
  }

  public logNext(options: LogOptions, channel: MessagePackChannel<LogEntry>): Promise<void> {
    return invokeMessagePack('subversion_log_next', { id: this.id, options: options, channel })
  }

  public cat(options: CatOptions): Promise<CatResult> {
    return invokeMessagePack('subversion_cat', { id: this.id, options: options })
  }

  public urlFromPath(path: string): Promise<string> {
    return invokeMessagePack('subversion_url_from_path', { id: this.id, path: path })
  }

  public defaultWcVersion(): Promise<Version> {
    return invokeMessagePack('subversion_default_wc_version', { id: this.id })
  }

  public getWcRoot(path: string): Promise<string> {
    return invokeMessagePack('subversion_get_wc_root', { id: this.id, path: path })
  }

  // --- Working Copy ---

  public checkout(options: CheckoutOptions): Promise<number> {
    return invokeMessagePack('subversion_checkout', { id: this.id, options: options })
  }

  public add(options: AddOptions): Promise<void> {
    return invokeMessagePack('subversion_add', { id: this.id, options: options })
  }

  public delete(options: DeleteOptions): Promise<DeleteResult> {
    return invokeMessagePack('subversion_delete', { id: this.id, options: options })
  }

  public revert(options: RevertOptions): Promise<void> {
    return invokeMessagePack('subversion_revert', { id: this.id, options: options })
  }

  public update(options: UpdateOptions): Promise<(number | null)[]> {
    return invokeMessagePack('subversion_update', { id: this.id, options: options })
  }

  public upgrade(options: UpgradeOptions): Promise<UpgradeResult> {
    return invokeMessagePack('subversion_upgrade', { id: this.id, options: options })
  }

  public vacuum(options: VacuumOptions): Promise<void> {
    return invokeMessagePack('subversion_vacuum', { id: this.id, options: options })
  }

  public switch(options: SwitchOptions): Promise<number> {
    return invokeMessagePack('subversion_switch', { id: this.id, options: options })
  }

  public cleanup(options: CleanupOptions): Promise<void> {
    return invokeMessagePack('subversion_cleanup', { id: this.id, options: options })
  }

  public relocate(options: RelocateOptions): Promise<void> {
    return invokeMessagePack('subversion_relocate', { id: this.id, options: options })
  }

  // --- Commit ---

  public commit(options: CommitOptions): Promise<CommitResult> {
    return invokeMessagePack('subversion_commit', { id: this.id, options: options })
  }

  public copy(options: CopyOptions): Promise<CopyResult> {
    return invokeMessagePack('subversion_copy', { id: this.id, options: options })
  }

  public mkdir(options: MkdirOptions): Promise<MkdirResult> {
    return invokeMessagePack('subversion_mkdir', { id: this.id, options: options })
  }

  public move(options: MoveOptions): Promise<MoveResult> {
    return invokeMessagePack('subversion_move', { id: this.id, options: options })
  }

  // --- Remote ---

  public import(options: ImportOptions, filters?: string[]): Promise<ImportResult> {
    return invokeMessagePack('subversion_import', {
      id: this.id,
      options: options,
      filters: filters,
    })
  }

  public export(options: ExportOptions): Promise<number | null> {
    return invokeMessagePack('subversion_export', { id: this.id, options: options })
  }

  // --- Diff ---

  public difference(options: ClientDifferenceOptions): Promise<ClientDifferenceResult> {
    return invokeMessagePack('subversion_difference', { id: this.id, options: options })
  }

  public blame(options: BlameOptions): Promise<BlameResult> {
    return invokeMessagePack('subversion_blame', { id: this.id, options: options })
  }

  // --- Merge ---

  public merge(options: MergeOptions): Promise<void> {
    return invokeMessagePack('subversion_merge', { id: this.id, options: options })
  }

  public patch(options: PatchOptions): Promise<void> {
    return invokeMessagePack('subversion_patch', { id: this.id, options: options })
  }

  // --- Conflict ---

  public conflictWalk(options: ConflictWalkOptions): Promise<ConflictWalkResult> {
    return invokeMessagePack('subversion_conflict_walk', { id: this.id, options: options })
  }

  // --- Lock ---

  public lock(options: LockOptions): Promise<void> {
    return invokeMessagePack('subversion_lock', { id: this.id, options: options })
  }

  public unlock(options: UnlockOptions): Promise<void> {
    return invokeMessagePack('subversion_unlock', { id: this.id, options: options })
  }

  // --- Property ---

  public propertyGet(options: PropertyGetOptions): Promise<PropertyGetResult> {
    return invokeMessagePack('subversion_property_get', { id: this.id, options: options })
  }

  public propertyList(options: PropertyListOptions): Promise<PropertyListResult> {
    return invokeMessagePack('subversion_property_list', { id: this.id, options: options })
  }

  public propertySet(options: PropertySetOptions): Promise<void> {
    return invokeMessagePack('subversion_property_set', { id: this.id, options: options })
  }

  public revisionPropertyList(
    options: RevisionPropertyListOptions,
  ): Promise<RevisionPropertyListResult> {
    return invokeMessagePack('subversion_revision_property_list', { id: this.id, options: options })
  }

  public wcRevisionStatus(
    options: WorkingCopyRevisionStatusOptions,
  ): Promise<WorkingCopyRevisionStatusResult> {
    return invokeMessagePack('subversion_wc_revision_status', { id: this.id, options })
  }

  public wcGetReplacedFile(path: string): Promise<WcReplacedNode | null> {
    return invokeMessagePack('subversion_wc_get_replaced_file', { id: this.id, path })
  }

  public raGetLocations(
    url: string,
    revision: number,
    locationRevisions: number[],
  ): Promise<Record<number, string>> {
    return invokeMessagePack('subversion_ra_get_locations', {
      id: this.id,
      url,
      revision,
      locationRevisions,
    })
  }

  public raGetLatestRevisionNumber(url: string, path?: string): Promise<number> {
    return invokeMessagePack('subversion_ra_get_latest_revision_number', {
      id: this.id,
      url,
      path,
    })
  }

  public logCache(
    repository: string,
    pegRevision: Revision,
    url: string,
    path: string,
    limit: number,
    start?: number,
  ): Promise<LogEntry[]> {
    return invokeMessagePack('subversion_log_cache', {
      id: this.id,
      pegRevision,
      repository,
      url,
      path,
      start,
      limit,
    })
  }

  public logCacheReverse(
    repository: string,
    pegRevision: Revision,
    url: string,
    path: string,
    limit: number,
    start: number,
  ): Promise<LogEntry[]> {
    return invokeMessagePack('subversion_log_cache_reverse', {
      id: this.id,
      pegRevision,
      repository,
      url,
      path,
      start,
      limit,
    })
  }

  async destroy() {
    this.listeners = {}
    const id = this._id
    if (id < 0) {
      console.warn('This subversion context has been destroyed')
      return
    }
    this._id = -1
    await invokeMessagePack('subversion_destroy', { id })
  }

  public static async call<T extends unknown = unknown>({
    factory,
    call,
    onError,
  }: {
    factory: SubversionFactory
    call: (context: Subversion) => Promise<T>
    onError: (error: any) => T
  }): Promise<T> {
    let context: Subversion | null = null
    try {
      context = await factory.context()
      return await call(context)
    } catch (error) {
      return onError(error)
      // console.warn('Subversion context error: ', error)
      // if (onError) {
      //   onError(error)
      // } else {
      //   Toast.error({
      //     content: `Error: ${errroHumanString(error)}`,
      //     stack: true,
      //   })
      // }
      // return { error }
    } finally {
      if (context) {
        factory.release(context)
      }
    }
  }

  public static async callOnce<T extends unknown = unknown>({
    factory,
    call,
    onError,
    onFinally,
  }: {
    factory: SubversionFactory
    call: (context: Subversion) => Promise<T>
    onError?: (error: any, handler: () => void) => void
    onFinally?: (context: Subversion | null) => void
  }): Promise<T | null> {
    let context: Subversion | null = null
    try {
      context = await factory.context()
      return await call(context)
    } catch (error) {
      console.warn('Subversion context error: ', error)
      const handler = () => {
        Toast.error({
          content: errorHumanString(error),
          stack: true,
        })
      }
      if (onError) {
        onError(error, handler)
      } else {
        handler()
      }
      return null
    } finally {
      if (onFinally) {
        onFinally(context)
      }
      if (context) {
        factory.release(context)
      }
    }
  }
  // check() {
  //   if (this.count === 0) {
  //     this.destroy()
  //   }
  // }

  // release() {
  //   this.count -= 1;
  //   if (this.doCheck) {
  //     this.check()
  //   }
  // }

  // acquire() {
  //   this.count += 1;
  // }
}

export function createTransientSubversion(
  onCreated?: (subversion: Subversion) => void,
): SubversionFactory {
  let created: Subversion[] = []
  return {
    async context() {
      const subversion = await Subversion.create()
      created.push(subversion)
      onCreated?.(subversion)
      return subversion
    },
    release(subversion) {
      subversion.destroy()
      created = created.filter((i) => i !== subversion)
    },
    releaseAll() {
      for (let i of created) {
        i.destroy()
      }
      created = []
    },
  }
}

export function createSingletonSubversion(
  onCreated?: (subversion: Subversion) => void,
): SubversionFactory {
  let subversion: null | Promise<Subversion> | Subversion = null
  return {
    async context() {
      if (subversion === null) {
        subversion = Subversion.create()
        subversion = await subversion
        onCreated?.(subversion)
        return subversion
      } else if (subversion instanceof Promise) {
        subversion = await subversion
        return subversion
      } else if (subversion instanceof Subversion) {
        return subversion
      } else {
        throw new Error('Unreachable')
      }
      // if (id > 0) {
      //   return id;
      // }
      // id = await context();
      // return id;
    },
    release() {},
    releaseAll() {
      async function call() {
        if (subversion === null) {
          return
        }
        if (subversion instanceof Promise) {
          subversion = await subversion
        }
        subversion.destroy()
        subversion = null
      }
      call()
    },
  }
}

export const SubverionContext = createContext<SubversionFactory | null>(null)

export function useSubversion(): SubversionFactory {
  const context = useContext(SubverionContext)

  if (context === null) {
    throw new Error('No subversion context provided')
  }

  return context
}
