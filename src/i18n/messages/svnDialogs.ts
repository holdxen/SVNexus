import { defineMessages } from '../define'

/**
 * SVN 常用命令对话框（添加/检出/提交/更新/切换/加锁等）里独有的文案。
 * 命令名、字段标签、选项文案、校验提示等复用词已在 shared.ts，这里只放各对话框自己的标题、进度和提示。
 */
export const svnDialogs = defineMessages({
  zh: {
    'svnDialogs.add.failed': '添加文件失败（{path}）：{error}',

    'svnDialogs.checkout.progressTitle': '正在检出…',
    'svnDialogs.checkout.descriptionUrl': '地址',
    'svnDialogs.checkout.descriptionPath': '路径',
    'svnDialogs.checkout.descriptionRevision': '版本',
    'svnDialogs.checkout.current': '当前：{name}',
    'svnDialogs.checkout.percentage': '百分比：{percent}%',
    'svnDialogs.checkout.unknown': '未知',
    'svnDialogs.checkout.failed': '检出失败：{error}',

    'svnDialogs.commit.select': '选择',
    'svnDialogs.commit.actual': '实际',
    'svnDialogs.commit.successAt': '提交成功（版本 {revision}）',

    'svnDialogs.update.success': '{path} 更新成功',
    'svnDialogs.update.successAt': '{path} 已更新到 r{revision}',
    'svnDialogs.update.failed': '更新失败：{error}',

    'svnDialogs.switch.success': '已切换到 r{revision}',
    'svnDialogs.switch.failed': '切换失败：{error}',

    'svnDialogs.lock.action': '操作：',
  },
  en: {
    'svnDialogs.add.failed': 'Failed to add file({path}): {error}',

    'svnDialogs.checkout.progressTitle': 'Checkout...',
    'svnDialogs.checkout.descriptionUrl': 'Url',
    'svnDialogs.checkout.descriptionPath': 'Path',
    'svnDialogs.checkout.descriptionRevision': 'Revision',
    'svnDialogs.checkout.current': 'Current: {name}',
    'svnDialogs.checkout.percentage': 'Percentage:{percent}%',
    'svnDialogs.checkout.unknown': 'unknown',
    'svnDialogs.checkout.failed': 'Failed to checkout: {error}',

    'svnDialogs.commit.select': 'Select',
    'svnDialogs.commit.actual': 'Actual',
    'svnDialogs.commit.successAt': 'Committed successfully at (revision {revision})',

    'svnDialogs.update.success': 'Update {path} successfully',
    'svnDialogs.update.successAt': 'Update {path} successfully at r{revision}',
    'svnDialogs.update.failed': 'Failed to update: {error}',

    'svnDialogs.switch.success': 'Switched to r{revision}',
    'svnDialogs.switch.failed': 'Failed to switch: {error}',

    'svnDialogs.lock.action': 'Action:',
  },
})
