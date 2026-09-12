/**
 * SingleTaskQueue - 单任务队列
 *
 * 保证同一时刻只有一个任务在执行；新任务入队时可以选择取消前面正在排队/执行的任务。
 *
 * 与原 C# 版本的主要差异：
 * 1. JS/TS 单线程执行，不存在 UI 线程校验（VerifyAccess），故已移除。
 * 2. Channel<T> 用数组队列 + 异步循环模拟。
 * 3. CancellationTokenSource -> AbortController；CancellationToken -> AbortSignal。
 * 4. TaskCompletionSource<T> -> 自定义 Deferred<T>。
 * 5. Action<CancellationToken> / Func<CancellationToken, Task> 两个重载合并为一个
 *    TaskFn 类型，运行时通过返回值是否为 Promise 来判断是否需要 await。
 * 6. 没有可靠的对象终结器（析构函数），因此不提供 Finalizer，只提供 dispose()。
 */

type TaskFn = (signal: AbortSignal) => void | Promise<void>

class Deferred<T = void> {
  readonly promise: Promise<T>
  resolve!: (value: T | PromiseLike<T>) => void
  reject!: (reason?: unknown) => void

  constructor() {
    this.promise = new Promise<T>((res, rej) => {
      this.resolve = res
      this.reject = rej
    })
  }
}

interface TaskMessage {
  controller: AbortController
  task: TaskFn
  finished: () => void
  deferred?: Deferred<void>
}

export class SingleTaskQueue {
  /** 任务队列，模拟原来的 Channel<TaskMessage> */
  private queue: TaskMessage[] = []

  /** 是否已有消费循环在跑，模拟 “TaskQueue is null” 的判断 */
  private processing = false

  /** 队列为空时触发；可返回 Promise 用于异步等待（对应原来的 event Func<Task?> QueueEmpty） */
  public onQueueEmpty?: () => Promise<void> | void

  /** 当前挂起的取消令牌集合 */
  private tokens: AbortController[] = []

  /** 默认是否取消前面的任务，对应 Single 属性 */
  public single = true

  private add(task: TaskFn, deferred?: Deferred<void>, cancelOthers = true) {
    const controller = new AbortController()
    const prevTokens = this.tokens
    this.tokens = [controller]

    const message: TaskMessage = {
      controller,
      task,
      finished: () => {
        const idx = this.tokens.indexOf(controller)
        if (idx >= 0) {
          this.tokens.splice(idx, 1)
        }
      },
      deferred,
    }

    this.queue.push(message)
    this.ensureProcessing()

    if (cancelOthers) {
      for (const t of prevTokens) {
        t.abort()
      }
    }
  }

  private ensureProcessing(): void {
    if (this.processing) {
      return
    }
    this.processing = true
    // 对应原来 Dispatcher.UIThread.InvokeAsync(async () => { ... })
    void this.processLoop()
  }

  private async processLoop(): Promise<void> {
    while (true) {
      const msg = this.queue.shift()
      if (!msg) {
        const awaiter = this.onQueueEmpty?.()
        if (awaiter) {
          await awaiter
        }
        this.processing = false
        return
      }

      try {
        const result = msg.task(msg.controller.signal)
        if (result instanceof Promise) {
          await result
        }
        msg.deferred?.resolve()
      } catch (ex) {
        if (msg.controller.signal.aborted) {
          console.info('Task is cancelled')
          msg.deferred?.reject(new DOMException('Aborted', 'AbortError'))
        } else {
          console.error(`Task execution failed, ${ex}`)
          msg.deferred?.reject(ex)
        }
      } finally {
        msg.finished()
      }
    }
  }

  /** 运行任务并等待其执行完成（会等待队列真正调度到该任务并跑完） */
  public async runAndWait(task: TaskFn, cancelOthers?: boolean): Promise<void> {
    const deferred = new Deferred<void>()
    this.add(task, deferred, cancelOthers ?? this.single)
    await deferred.promise
  }

  /** 运行任务，仅等待入队完成，不等待任务真正执行结束 */
  public run(task: TaskFn, cancelOthers?: boolean) {
    return this.add(task, undefined, cancelOthers ?? this.single)
  }

  /** 释放资源：取消所有挂起任务、清空队列 */
  public dispose(): void {
    for (const t of this.tokens) {
      t.abort()
    }
    this.tokens = []
    this.queue = []
    this.processing = false
  }
}
