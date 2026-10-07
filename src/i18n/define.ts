/**
 * 每个功能模块一个消息分片，中英对照写在同一个文件里。
 * `defineMessages` 会用类型强制两边 key 完全一致：少了 key 会报错，多了 key 也会报错。
 */
export function defineMessages<T extends Record<string, string>>(messages: {
  zh: T
  en: { [K in keyof T]: string }
}): { zh: T; en: { [K in keyof T]: string } } {
  return messages
}
