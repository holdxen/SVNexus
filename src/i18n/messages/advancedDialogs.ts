import { defineMessages } from '../define'

/**
 * 高级对话框词表：差异、导出、导入、补丁、合并、认证、SSL 与加载弹窗。
 * 通用字段名、选项文案与校验提示复用 shared，这里只放各对话框特有的文案。
 */
export const advancedDialogs = defineMessages({
  zh: {
    'advancedDialogs.progress.current': '当前：{name}',
    'advancedDialogs.progress.percentage': '百分比：{percent}%',
    'advancedDialogs.progress.unknown': '未知',
    'advancedDialogs.progress.path': '路径',
    'advancedDialogs.progress.url': '地址',
    'advancedDialogs.progress.from': '从',
    'advancedDialogs.progress.to': '到',

    'advancedDialogs.difference.title': '差异',
    'advancedDialogs.difference.display.both': '全部',
    'advancedDialogs.difference.display.contentOnly': '仅内容',
    'advancedDialogs.difference.display.propertyOnly': '仅属性',
    'advancedDialogs.difference.relate.none': '不关联',
    'advancedDialogs.difference.relate.path': '当前路径',
    'advancedDialogs.difference.relate.root': '仓库根目录',

    'advancedDialogs.export.progressTitle': '正在导出…',
    'advancedDialogs.export.failed': '导出失败：{error}',
    'advancedDialogs.export.eolLf': 'LF（Unix/macOS）',
    'advancedDialogs.export.eolCrlf': 'CRLF（Windows）',

    'advancedDialogs.import.progressTitle': '正在导入…',
    'advancedDialogs.import.failed': '导入失败：{error}',
    'advancedDialogs.import.confirmTitle': '是否导入',
    'advancedDialogs.import.kind': '类型：',
    'advancedDialogs.import.size': '大小：',
    'advancedDialogs.import.modifyTime': '修改时间：',
    'advancedDialogs.import.filterNone': '无',
    'advancedDialogs.import.filterOneByOne': '逐个确认',
    'advancedDialogs.import.filterLines': '按行过滤',

    'advancedDialogs.merge.modeRange': '合并版本范围',
    'advancedDialogs.merge.modeTrees': '合并两棵不同的树',

    'advancedDialogs.authenticate.title': '认证',
    'advancedDialogs.authenticate.passwordMethod': '密码',
    'advancedDialogs.authenticate.keyMethod': '密钥',

    'advancedDialogs.sshPassphrase.title': '口令',
    'advancedDialogs.sshPassphrase.incorrect': '口令不正确，请重试',
    'advancedDialogs.sshPassphrase.protected': '该密钥受口令保护。',
    'advancedDialogs.sshPassphrase.usernameLine': '用户名：{username}',

    'advancedDialogs.sslTrust.hostname': '主机名：',
    'advancedDialogs.sslTrust.fingerprint': '指纹：',
    'advancedDialogs.sslTrust.validFrom': '生效时间：',
    'advancedDialogs.sslTrust.validUntil': '失效时间：',
    'advancedDialogs.sslTrust.issuer': '颁发者：',
    'advancedDialogs.sslTrust.asciiCert': 'ASCII 证书：',
  },
  en: {
    'advancedDialogs.progress.current': 'Current: {name}',
    'advancedDialogs.progress.percentage': 'Percentage:{percent}%',
    'advancedDialogs.progress.unknown': 'unknown',
    'advancedDialogs.progress.path': 'Path',
    'advancedDialogs.progress.url': 'URL',
    'advancedDialogs.progress.from': 'From',
    'advancedDialogs.progress.to': 'To',

    'advancedDialogs.difference.title': 'Difference',
    'advancedDialogs.difference.display.both': 'Both',
    'advancedDialogs.difference.display.contentOnly': 'ContentOnly',
    'advancedDialogs.difference.display.propertyOnly': 'PropertyOnly',
    'advancedDialogs.difference.relate.none': 'None',
    'advancedDialogs.difference.relate.path': 'Path',
    'advancedDialogs.difference.relate.root': 'Root',

    'advancedDialogs.export.progressTitle': 'Export...',
    'advancedDialogs.export.failed': 'Failed to export: {error}',
    'advancedDialogs.export.eolLf': 'LF (Unix/macOS)',
    'advancedDialogs.export.eolCrlf': 'CRLF (Windows)',

    'advancedDialogs.import.progressTitle': 'Import...',
    'advancedDialogs.import.failed': 'Failed to import: {error}',
    'advancedDialogs.import.confirmTitle': 'Import or not',
    'advancedDialogs.import.kind': 'Kind:',
    'advancedDialogs.import.size': 'Size:',
    'advancedDialogs.import.modifyTime': 'Modify time:',
    'advancedDialogs.import.filterNone': 'none',
    'advancedDialogs.import.filterOneByOne': 'one by one',
    'advancedDialogs.import.filterLines': 'lines',

    'advancedDialogs.merge.modeRange': 'Merge a range of revisions',
    'advancedDialogs.merge.modeTrees': 'Merge two different trees',

    'advancedDialogs.authenticate.title': 'Authenticate',
    'advancedDialogs.authenticate.passwordMethod': 'Password',
    'advancedDialogs.authenticate.keyMethod': 'Key',

    'advancedDialogs.sshPassphrase.title': 'Passphrase',
    'advancedDialogs.sshPassphrase.incorrect': 'Incorrect passphrase, please try again',
    'advancedDialogs.sshPassphrase.protected': 'This key is protected by a passphrase.',
    'advancedDialogs.sshPassphrase.usernameLine': 'Username: {username}',

    'advancedDialogs.sslTrust.hostname': 'Hostname:',
    'advancedDialogs.sslTrust.fingerprint': 'Fingerprint:',
    'advancedDialogs.sslTrust.validFrom': 'ValidFrom:',
    'advancedDialogs.sslTrust.validUntil': 'ValidUntil:',
    'advancedDialogs.sslTrust.issuer': 'Issuer:',
    'advancedDialogs.sslTrust.asciiCert': 'AsciiCert:',
  },
})
