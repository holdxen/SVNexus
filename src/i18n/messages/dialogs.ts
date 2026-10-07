import { defineMessages } from '../define'

export const dialogs = defineMessages({
  zh: {
    'dialogs.saveFile': '保存文件',
    'dialogs.openFromPath': '从路径打开',
    'dialogs.update.found': '发现新版本 v{version}',
    'dialogs.update.description':
      '当前版本 v{current}，发现新版本 v{version}，更新将在下载完成后自动安装并重启应用。',
    'dialogs.update.downloadProgress': '下载进度:',
    'dialogs.update.downloading': '下载中…',
    'dialogs.update.updateNow': '立即更新',
    'dialogs.update.releaseNotes': '在 GitHub 查看更新说明',
  },
  en: {
    'dialogs.saveFile': 'Save file',
    'dialogs.openFromPath': 'Open from path',
    'dialogs.update.found': 'Version v{version} is available',
    'dialogs.update.description':
      'You are on v{current}; version v{version} is available. The update will install automatically and restart the app once the download finishes.',
    'dialogs.update.downloadProgress': 'Download progress:',
    'dialogs.update.downloading': 'Downloading…',
    'dialogs.update.updateNow': 'Update now',
    'dialogs.update.releaseNotes': 'View release notes on GitHub',
  },
})
