import { defineMessages } from '../define'

export const merge = defineMessages({
  zh: {
    'merge.description':
      '将仓库中的变更应用到目标工作副本：先选择合并方式，再指定变更来源与合并目标。',
    'merge.mode.title': '合并方式',
    'merge.mode.rangeHint': '将来源分支在指定版本范围内的变更应用到目标，是最常用的合并方式。',
    'merge.mode.treesHint': '比较两个来源的差异，并把差异应用到目标，常用于分支同步或反向合并。',
    'merge.source.title': '合并来源',
    'merge.source.placeholder': '仓库 URL 或本地路径',
    'merge.source.pegTitle': 'Peg (可选，来源路径曾改名时才需要):',
    'merge.source.rangeHint': '起始版本之后、直到结束版本的变更将被合并到目标。',
    'merge.source.olderTreePlaceholder': '较旧树的 URL 或路径',
    'merge.source.newerTreePlaceholder': '较新树的 URL 或路径',
    'merge.source.source1Title': 'Source 1 (比较基准):',
    'merge.source.source2Title': 'Source 2 (比较目标):',
    'merge.source.treesHint': 'Source 1 到 Source 2 之间的差异将被应用到目标。',
    'merge.target.title': '合并目标',
    'merge.target.pathTitle': 'Target (接受合并的工作副本):',
    'merge.advanced.title': '高级选项',
    'merge.advanced.ignoreMergeInfoHint': '忽略已合并记录，允许重复合并相同变更',
    'merge.advanced.ignoreAncestryHint': '忽略血缘关系，仅按内容差异比较',
    'merge.advanced.forceDeleteHint': '合并时强制删除存在本地修改的文件',
    'merge.advanced.recordOnlyHint': '仅记录合并信息，不修改工作副本内容',
    'merge.advanced.dryRunHint': '试运行，只预览将产生的变更而不实际修改',
    'merge.advanced.allowMixedRevisionHint': '允许工作副本处于混合版本时操作',
  },
  en: {
    'merge.description':
      'Apply changes from a repository to a target working copy: first choose a merge mode, then specify the change source and the merge target.',
    'merge.mode.title': 'Merge mode',
    'merge.mode.rangeHint':
      'Apply the changes made on the source branch within a revision range to the target; this is the most common way to merge.',
    'merge.mode.treesHint':
      'Compare two sources and apply the difference to the target; commonly used to sync branches or to merge in reverse.',
    'merge.source.title': 'Merge source',
    'merge.source.placeholder': 'Repository URL or local path',
    'merge.source.pegTitle': 'Peg (optional, only needed if the source path was renamed):',
    'merge.source.rangeHint':
      'Changes made after the start revision, up to and including the end revision, will be merged into the target.',
    'merge.source.olderTreePlaceholder': 'URL or path of the older tree',
    'merge.source.newerTreePlaceholder': 'URL or path of the newer tree',
    'merge.source.source1Title': 'Source 1 (base for comparison):',
    'merge.source.source2Title': 'Source 2 (target for comparison):',
    'merge.source.treesHint':
      'The difference between Source 1 and Source 2 will be applied to the target.',
    'merge.target.title': 'Merge target',
    'merge.target.pathTitle': 'Target (working copy that receives the merge):',
    'merge.advanced.title': 'Advanced options',
    'merge.advanced.ignoreMergeInfoHint':
      'Ignore the record of what has already been merged, allowing the same changes to be merged again',
    'merge.advanced.ignoreAncestryHint': 'Ignore ancestry and compare by content differences only',
    'merge.advanced.forceDeleteHint':
      'Force deletion of files that have local modifications during the merge',
    'merge.advanced.recordOnlyHint':
      'Record merge information only; do not modify working copy content',
    'merge.advanced.dryRunHint':
      'Dry run: preview the changes that would be produced without making them',
    'merge.advanced.allowMixedRevisionHint':
      'Allow the operation while the working copy is at mixed revisions',
  },
})
