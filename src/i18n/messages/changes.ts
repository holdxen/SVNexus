import { defineMessages } from '../define'

/**
 * 变更视图、历史视图、远端视图与工作区外框的界面词表。
 * SVN 命令名、字段标签、状态名等跨模块词条统一放在 shared.ts，这里只收本模块独有的文案。
 */
export const changes = defineMessages({
  zh: {
    'changes.tree.all': '全部',
    'changes.ignore.extension': '忽略 *.{extension}',
    'changes.ignore.extensionRecursively': '递归忽略 *.{extension}',
    'changes.ignore.name': '忽略 {name}',
    'changes.ignore.nameRecursively': '递归忽略 {name}',

    'changes.list.failedToGetStatus': '获取状态失败：{error}',

    'changes.property.confirmDeletePrefix': '确定要删除属性',
    'changes.property.confirmDeleteSuffix': '（位于 {target}）？',
    'changes.property.confirmRevert': '确定要还原属性 {name} 吗？',
    'changes.property.copyName': '复制属性名',
    'changes.property.copyOriginalValue': '复制原属性值',
    'changes.property.copyModifiedValue': '复制修改后的属性值',
    'changes.error.unexpected': '发生意外错误：{error}',
    'changes.error.notUnderVersionControl': '“{path}” 不在版本控制之下',
    'changes.error.emptyWorkingCopy': '工作副本为空',

    'changes.history.failedToQueryRepository': '无法查询 {path} 的仓库信息',
    'changes.history.copyMessage': '复制提交信息',
    'changes.history.copyAuthor': '复制作者',
    'changes.history.copyRevision': '复制版本号',
    'changes.history.copyDate': '复制日期',
    'changes.history.mergeReset': '合并（重置）',
    'changes.history.tabDetail': '详情',
    'changes.history.tabChanges': '变更',
    'changes.history.tabSnapshot': '快照',

    'changes.snapshot.saveSuccess': '{path} 保存成功',
    'changes.snapshot.saveFailed': '{path} 保存失败：{error}',

    'changes.remote.notWorkingCopyOfRepository': '{path} 不是仓库的工作副本',

    'changes.navigation.emptyFolder': '{title}（空）',

    'changes.workspace.questionTitle': '询问',
    'changes.workspace.errorTitle': '错误',
    'changes.workspace.upgradeTitle': '升级',
    'changes.workspace.alreadyOpened': '{path} 已经打开\n是否切换到该视图？',
    'changes.workspace.untitled': '未命名',
  },
  en: {
    'changes.tree.all': 'All',
    'changes.ignore.extension': 'Ignore *.{extension}',
    'changes.ignore.extensionRecursively': 'Ignore *.{extension} recursively',
    'changes.ignore.name': 'Ignore {name}',
    'changes.ignore.nameRecursively': 'Ignore {name} recursively',

    'changes.list.failedToGetStatus': 'Failed to get status: {error}',

    'changes.property.confirmDeletePrefix': 'Are you sure to delete property',
    'changes.property.confirmDeleteSuffix': ' on {target} ?',
    'changes.property.confirmRevert': 'Are you sure to revert property {name}',
    'changes.property.copyName': 'Copy property name',
    'changes.property.copyOriginalValue': 'Copy original property value',
    'changes.property.copyModifiedValue': 'Copy modified property value',
    'changes.error.unexpected': 'Unexpected error {error}',
    'changes.error.notUnderVersionControl': "'{path}' is not under version control",
    'changes.error.emptyWorkingCopy': 'Empty working copy',

    'changes.history.failedToQueryRepository': 'Failed to query repository of {path}',
    'changes.history.copyMessage': 'Copy message',
    'changes.history.copyAuthor': 'Copy author',
    'changes.history.copyRevision': 'Copy revision',
    'changes.history.copyDate': 'Copy date',
    'changes.history.mergeReset': 'Merge(reset)',
    'changes.history.tabDetail': 'Detail',
    'changes.history.tabChanges': 'Changes',
    'changes.history.tabSnapshot': 'Snapshot',

    'changes.snapshot.saveSuccess': 'Save {path} successfully',
    'changes.snapshot.saveFailed': 'Save {path} failed: {error}',

    'changes.remote.notWorkingCopyOfRepository': '{path} is not a working copy of a repository',

    'changes.navigation.emptyFolder': '{title}(empty)',

    'changes.workspace.questionTitle': 'Question',
    'changes.workspace.errorTitle': 'Error',
    'changes.workspace.upgradeTitle': 'Upgrade',
    'changes.workspace.alreadyOpened': '{path} has already been opened\nWhether go to the view',
    'changes.workspace.untitled': 'Untitled',
  },
})
