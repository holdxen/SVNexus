import { readFile, remove } from '@tauri-apps/plugin-fs'

import { logsPackage } from '@/context/Functions'
import { t } from '@/i18n'
import type { FeedbackReport } from '@/views/dialogs/FeedbackDialog'

/**
 * 反馈提交的目标端点，由构建模式决定（见仓库根的 .env.development / .env.production）。
 *
 * 开发和正式构建写的是两个不同的库：开发指向本地 wrangler dev（本地库），正式指向已部署的
 * Worker（线上库）。所以「开发环境提交成功、线上看不到」是正常的，不是丢数据——要验线上
 * 链路得跑正式构建，或者用 .env.development.local 临时把开发端点也指过去。
 */
export const FEEDBACK_ENDPOINT: string = import.meta.env.VITE_FEEDBACK_ENDPOINT ?? ''

/**
 * 附件与日志上报暂时关闭：D1 免费版单库上限 500MB，放不下二进制。
 * 实现全部保留（读文件、压缩图片、打包日志都在），以后接上 R2 把这个改成 true 即可。
 *
 * 标成 boolean 而不是字面量 false —— 否则下面整段会被判定为不可达代码。
 */
export const ATTACHMENTS_ENABLED: boolean = false

const TIMEOUT_MS = 60_000

/** 与后端 MAX_FILE_BYTES 保持一致 */
const MAX_FILE_BYTES = 1_500_000
/** 图片压缩的目标体积，留出余量以免卡在上限边缘 */
const TARGET_IMAGE_BYTES = 1_000_000

/** 压缩时依次尝试的长边与质量，先给最好的，不行再降 */
const IMAGE_STEPS: [number, number][] = [
  [2400, 0.85],
  [2400, 0.7],
  [1800, 0.7],
  [1800, 0.5],
  [1400, 0.5],
  [1000, 0.5],
]

/**
 * 把一份反馈连同附件提交到后端。成功即返回；失败抛出可直接展示给用户的错误信息。
 */
export async function submitFeedback(report: FeedbackReport, paths: string[]): Promise<void> {
  if (!FEEDBACK_ENDPOINT) {
    throw new Error(t('feedback.submit.notConfigured'))
  }

  const form = new FormData()
  // 只提交后端用得上的字段：附件走 files，本地路径不进请求体
  form.append(
    'report',
    JSON.stringify({
      category: report.category,
      content: report.content,
      steps: report.steps,
      contact: report.contact,
      diagnostics: report.diagnostics,
    }),
  )
  if (ATTACHMENTS_ENABLED) {
    for (const path of paths) {
      const file = await loadAttachment(path)
      form.append('files', toBlob(file.bytes, file.type), file.name)
    }
    if (report.includeLogs) {
      form.append('files', await logArchive(), LOG_ARCHIVE_NAME)
    }
  }

  const controller = new AbortController()
  const timeout = setTimeout(() => controller.abort(), TIMEOUT_MS)

  try {
    // 不手动设置 Content-Type：必须让浏览器带上 multipart 的 boundary
    const response = await fetch(FEEDBACK_ENDPOINT, {
      method: 'POST',
      body: form,
      signal: controller.signal,
    })

    if (!response.ok) {
      // 被拒绝的请求会返回 {"error": "..."}，拿不到就退回状态码
      const reason = await response
        .json()
        .then((body: { error?: string }) => body?.error)
        .catch(() => undefined)
      throw new Error(reason ?? `HTTP ${response.status}`)
    }
  } catch (error) {
    if (error instanceof Error && error.name === 'AbortError') {
      throw new Error(t('feedback.submit.timeout'))
    }
    // 网络不可用或被 CORS 拦下时 fetch 只抛 TypeError，其信息是浏览器原生的英文
    if (error instanceof TypeError) {
      throw new Error(t('feedback.submit.unreachable'))
    }
    throw error
  } finally {
    clearTimeout(timeout)
  }
}

/**
 * TS 6 把 `Uint8Array` 参数化成 `Uint8Array<ArrayBufferLike>`，而 `BlobPart` 只接受具体的
 * `ArrayBuffer` 视图。复制一份即可，附件本来就在 1.5MB 以内。
 */
function toBlob(bytes: Uint8Array, type: string): Blob {
  return new Blob([new Uint8Array(bytes)], { type })
}

const LOG_ARCHIVE_NAME = 'feedback-logs.zip'

/**
 * 让后端把日志目录打成压缩包。归档只是中转过一道，读进内存后立刻删掉——日志目录
 * 可能有几十兆，没必要留在磁盘上。
 */
async function logArchive(): Promise<Blob> {
  const path = await logsPackage()
  try {
    return toBlob(await readFile(path), 'application/zip')
  } finally {
    await remove(path).catch(() => undefined)
  }
}

interface UploadFile {
  name: string
  type: string
  bytes: Uint8Array
}

async function loadAttachment(path: string): Promise<UploadFile> {
  const name = path.split(/[\\/]/).pop() || 'attachment'

  let bytes: Uint8Array
  try {
    bytes = await readFile(path)
  } catch {
    throw new Error(t('feedback.submit.unreadable', { name }))
  }

  const type = guessContentType(name)
  if (!isCompressible(type)) {
    if (bytes.byteLength > MAX_FILE_BYTES) {
      throw new Error(t('feedback.submit.fileTooLarge', { name }))
    }
    return { name, type, bytes }
  }

  const compressed = await compressImage(bytes, type)
  if (compressed === null) {
    throw new Error(t('feedback.submit.imageTooLarge', { name }))
  }
  // 转成 JPEG 后扩展名要跟着改，否则接收方按后缀打开会失败
  return { name: name.replace(/\.[^.]*$/, '') + '.jpg', type: 'image/jpeg', bytes: compressed }
}

async function compressImage(bytes: Uint8Array, type: string): Promise<Uint8Array | null> {
  if (bytes.byteLength <= TARGET_IMAGE_BYTES) {
    return bytes
  }

  const image = await decode(bytes, type)
  if (image === null) {
    // 解不开就按原样传，前提是还没超上限
    return bytes.byteLength <= MAX_FILE_BYTES ? bytes : null
  }

  for (const [maxEdge, quality] of IMAGE_STEPS) {
    const blob = await render(image, maxEdge, quality)
    if (blob && blob.size <= TARGET_IMAGE_BYTES) {
      return new Uint8Array(await blob.arrayBuffer())
    }
  }
  return null
}

function decode(bytes: Uint8Array, type: string): Promise<HTMLImageElement | null> {
  return new Promise((resolve) => {
    const url = URL.createObjectURL(toBlob(bytes, type))
    const image = new Image()
    image.onload = () => {
      URL.revokeObjectURL(url)
      resolve(image)
    }
    image.onerror = () => {
      URL.revokeObjectURL(url)
      resolve(null)
    }
    image.src = url
  })
}

function render(image: HTMLImageElement, maxEdge: number, quality: number): Promise<Blob | null> {
  const source = Math.max(image.naturalWidth, image.naturalHeight)
  const scale = source > 0 ? Math.min(1, maxEdge / source) : 1
  const canvas = document.createElement('canvas')
  canvas.width = Math.max(1, Math.round(image.naturalWidth * scale))
  canvas.height = Math.max(1, Math.round(image.naturalHeight * scale))

  const context = canvas.getContext('2d')
  if (!context) {
    return Promise.resolve(null)
  }
  // JPEG 没有透明通道，不铺底的话 PNG 的透明区域会变成黑色
  context.fillStyle = '#ffffff'
  context.fillRect(0, 0, canvas.width, canvas.height)
  context.drawImage(image, 0, 0, canvas.width, canvas.height)

  return new Promise((resolve) => canvas.toBlob(resolve, 'image/jpeg', quality))
}

function guessContentType(name: string): string {
  const extension = name.split('.').pop()?.toLowerCase() ?? ''
  switch (extension) {
    case 'png':
      return 'image/png'
    case 'jpg':
    case 'jpeg':
      return 'image/jpeg'
    case 'webp':
      return 'image/webp'
    case 'gif':
      return 'image/gif'
    case 'bmp':
      return 'image/bmp'
    case 'txt':
    case 'log':
      return 'text/plain'
    case 'json':
      return 'application/json'
    case 'zip':
      return 'application/zip'
    default:
      return 'application/octet-stream'
  }
}

/** SVG 画进 canvas 可能污染画布，当作普通文件处理 */
function isCompressible(type: string): boolean {
  return ['image/png', 'image/jpeg', 'image/webp', 'image/gif', 'image/bmp'].includes(type)
}
