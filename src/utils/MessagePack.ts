import { decode, encode } from '@msgpack/msgpack'
import { Channel as TauriChannel, invoke, SERIALIZE_TO_IPC_FN } from '@tauri-apps/api/core'

/**
 * 调用后端 messagepack_command 标记的函数，使用 MessagePack 编码。
 *
 * @param cmd - 命令名称（对应 Rust 函数名）
 * @param args - 参数对象（对应函数的参数），可选，默认为空对象
 * @returns 返回反序列化后的结果
 *
 * @example
 * ```ts
 * // 后端：
 * // #[messagepack_command]
 * // fn get_user(name: String, age: i32) -> Result<User, Error> { ... }
 *
 * // 前端：
 * const user = await invokeMessagePack<User>("get_user", { name: "Alice", age: 30 });
 * ```
 */
export async function invokeMessagePack<T>(
  cmd: string,
  args: Record<string, unknown> = {},
): Promise<T> {
  const requestBytes = encode(encodeMessagePackValue(args))

  // 通过 Tauri invoke 发送
  // tauri::ipc::Request 的 body 字段是 Vec<u8>，对应前端的 Uint8Array
  const response: Uint8Array = await invoke(cmd, requestBytes)

  // 反序列化响应
  return decode(response) as T
}

function encodeMessagePackValue(value: unknown): unknown {
  if (value instanceof MessagePackChannel) {
    return `__CHANNEL__:${value.id}`
  }

  if (Array.isArray(value)) {
    return value.map(encodeMessagePackValue)
  }

  if (value && typeof value === 'object') {
    const prototype = Object.getPrototypeOf(value)
    if (prototype !== Object.prototype && prototype !== null) {
      return value
    }

    return Object.fromEntries(
      Object.entries(value).map(([key, nestedValue]) => [key, encodeMessagePackValue(nestedValue)]),
    )
  }

  return value
}

/**
 * MessagePack Channel —— 与 Tauri `Channel<T>` 相同 API，但消息走二进制序列化。
 *
 * ## 实现思路
 *
 * 直接复用原生 `Channel`：注册回调、保序、`pendingMessages`、`end` 处理、
 * `unregisterCallback` 等底层逻辑全部交给原生 Channel，本类只做一件事——
 * 接到字节数组后用 MessagePack 反序列化并直接派发给用户。
 *
 * ## 与原生 Channel 的区别
 *
 * - 原生 `Channel<T>`：后端 `channel.send(value)` 走 serde JSON
 * - `MessagePackChannel<T>`：后端先把数据编码成 MessagePack 字节，
 *   再 `channel.send(bytes)`，bytes 通过 Tauri 二进制通道传输，前端 BSON 反序列化拿到 T
 *
 * Tauri 后端 `channel.send(Vec<u8>)` 时，小 payload 走 `eval` 路径会把
 * `Uint8Array.buffer`（即 `ArrayBuffer`）传过来，大 payload 走 `fetch` 才是 `Uint8Array`，
 * 所以解码时统一归一化成 `Uint8Array`。
 *
 * ## 后端用法
 *
 * ```rust
 * use tauri::ipc::Channel;
 * #[messagepack_command]
 * fn subscribe(channel: Channel<Vec<u8>>) {
 *     let bytes = rmp_serde::to_vec_named(&MyEvent { ... }).unwrap();
 *     channel.send(bytes).unwrap();
 * }
 * ```
 *
 * ## 前端用法
 *
 * ```ts
 * const ch = new MessagePackChannel<MyEvent>((event) => {
 *   console.log('收到事件', event)
 * })
 * await invokeMessagePack('subscribe', { channel: ch })
 * ```
 */
export class MessagePackChannel<T = unknown> {
  /** 内部复用原生 Channel，由它负责注册回调、保序、清理等所有底层逻辑 */
  readonly #inner: TauriChannel<Uint8Array>

  /** 用户回调：接收反序列化后的 T */
  #handler: (response: T) => void

  constructor(onmessage?: (response: T) => void) {
    this.#handler = onmessage ?? (() => {})
    this.#inner = new TauriChannel<Uint8Array>((bytes) => {
      this.#dispatch(bytes)
    })
  }

  /** Tauri Channel 回调 id（与原生 Channel.id 语义一致） */
  get id(): number {
    return this.#inner.id
  }

  set onmessage(handler: (response: T) => void) {
    this.#handler = handler
  }

  get onmessage(): (response: T) => void {
    return this.#handler
  }

  /** 收到字节后 MessagePack 解码并派发给用户回调 */
  #dispatch(bytes: Uint8Array | ArrayBuffer): void {
    // Tauri 小 payload 走 eval 传的是 ArrayBuffer（Uint8Array.buffer），大 payload 才是 Uint8Array
    const u8 = bytes instanceof Uint8Array ? bytes : new Uint8Array(bytes)
    this.#handler(decode(u8) as T)
  }

  /** Tauri IPC 序列化钩子，转交给内部 Channel */
  [SERIALIZE_TO_IPC_FN](): string {
    return this.#inner[SERIALIZE_TO_IPC_FN]()
  }

  /** JSON.stringify 时按内部 Channel 的格式输出，保证 invoke 能识别 */
  toJSON(): string {
    return this.#inner.toJSON()
  }
}
