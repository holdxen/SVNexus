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
    'changes.history.copyUrl': '复制 URL',
    'changes.history.copyLocation': '复制仓库路径',
    'changes.history.mergeReset': '合并（重置）',
    'changes.history.tabDetail': '详情',
    'changes.history.tabChanges': '变更',
    'changes.history.tabSnapshot': '快照',
    'changes.history.saveSuccess': '{path} 保存成功',
    'changes.history.saveFailed': '{path} 保存失败：{error}',

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
    'changes.history.copyUrl': 'Copy URL',
    'changes.history.copyLocation': 'Copy repository path',
    'changes.history.mergeReset': 'Merge(reset)',
    'changes.history.tabDetail': 'Detail',
    'changes.history.tabChanges': 'Changes',
    'changes.history.tabSnapshot': 'Snapshot',
    'changes.history.saveSuccess': 'Save {path} successfully',
    'changes.history.saveFailed': 'Save {path} failed: {error}',

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
  es: {
    'changes.tree.all': 'Todo',
    'changes.ignore.extension': 'Ignorar *.{extension}',
    'changes.ignore.extensionRecursively': 'Ignorar *.{extension} de forma recursiva',
    'changes.ignore.name': 'Ignorar {name}',
    'changes.ignore.nameRecursively': 'Ignorar {name} de forma recursiva',

    'changes.list.failedToGetStatus': 'No se pudo obtener el estado: {error}',

    'changes.property.confirmDeletePrefix': '¿Seguro que deseas eliminar la propiedad',
    'changes.property.confirmDeleteSuffix': ' en {target}?',
    'changes.property.confirmRevert': '¿Seguro que deseas revertir la propiedad {name}?',
    'changes.property.copyName': 'Copiar nombre de propiedad',
    'changes.property.copyOriginalValue': 'Copiar valor original de la propiedad',
    'changes.property.copyModifiedValue': 'Copiar valor modificado de la propiedad',
    'changes.error.unexpected': 'Error inesperado: {error}',
    'changes.error.notUnderVersionControl': "'{path}' no está bajo control de versiones",
    'changes.error.emptyWorkingCopy': 'Copia de trabajo vacía',

    'changes.history.failedToQueryRepository': 'No se pudo consultar el repositorio de {path}',
    'changes.history.copyMessage': 'Copiar mensaje',
    'changes.history.copyAuthor': 'Copiar autor',
    'changes.history.copyRevision': 'Copiar revisión',
    'changes.history.copyDate': 'Copiar fecha',
    'changes.history.copyUrl': 'Copiar URL',
    'changes.history.copyLocation': 'Copiar ruta del repositorio',
    'changes.history.mergeReset': 'Fusión (reset)',
    'changes.history.tabDetail': 'Detalle',
    'changes.history.tabChanges': 'Cambios',
    'changes.history.tabSnapshot': 'Instantánea',
    'changes.history.saveSuccess': '{path} se guardó correctamente',
    'changes.history.saveFailed': 'No se pudo guardar {path}: {error}',

    'changes.snapshot.saveSuccess': '{path} se guardó correctamente',
    'changes.snapshot.saveFailed': 'No se pudo guardar {path}: {error}',

    'changes.remote.notWorkingCopyOfRepository':
      '{path} no es una copia de trabajo de un repositorio',

    'changes.navigation.emptyFolder': '{title} (vacía)',

    'changes.workspace.questionTitle': 'Pregunta',
    'changes.workspace.errorTitle': 'Error',
    'changes.workspace.upgradeTitle': 'Actualización',
    'changes.workspace.alreadyOpened': '{path} ya está abierto\n¿Quieres ir a esa vista?',
    'changes.workspace.untitled': 'Sin título',
  },
  pt: {
    'changes.tree.all': 'Tudo',
    'changes.ignore.extension': 'Ignorar *.{extension}',
    'changes.ignore.extensionRecursively': 'Ignorar *.{extension} recursivamente',
    'changes.ignore.name': 'Ignorar {name}',
    'changes.ignore.nameRecursively': 'Ignorar {name} recursivamente',

    'changes.list.failedToGetStatus': 'Falha ao obter o status: {error}',

    'changes.property.confirmDeletePrefix': 'Tem certeza de que deseja excluir a propriedade',
    'changes.property.confirmDeleteSuffix': ' em {target}?',
    'changes.property.confirmRevert': 'Tem certeza de que deseja reverter a propriedade {name}?',
    'changes.property.copyName': 'Copiar nome da propriedade',
    'changes.property.copyOriginalValue': 'Copiar valor original da propriedade',
    'changes.property.copyModifiedValue': 'Copiar valor modificado da propriedade',
    'changes.error.unexpected': 'Erro inesperado: {error}',
    'changes.error.notUnderVersionControl': "'{path}' não está sob controle de versão",
    'changes.error.emptyWorkingCopy': 'Cópia de trabalho vazia',

    'changes.history.failedToQueryRepository': 'Falha ao consultar o repositório de {path}',
    'changes.history.copyMessage': 'Copiar mensagem',
    'changes.history.copyAuthor': 'Copiar autor',
    'changes.history.copyRevision': 'Copiar revisão',
    'changes.history.copyDate': 'Copiar data',
    'changes.history.copyUrl': 'Copiar URL',
    'changes.history.copyLocation': 'Copiar caminho do repositório',
    'changes.history.mergeReset': 'Mesclagem (reset)',
    'changes.history.tabDetail': 'Detalhes',
    'changes.history.tabChanges': 'Alterações',
    'changes.history.tabSnapshot': 'Instantâneo',
    'changes.history.saveSuccess': '{path} salvo com sucesso',
    'changes.history.saveFailed': 'Falha ao salvar {path}: {error}',

    'changes.snapshot.saveSuccess': '{path} salvo com sucesso',
    'changes.snapshot.saveFailed': 'Falha ao salvar {path}: {error}',

    'changes.remote.notWorkingCopyOfRepository':
      '{path} não é uma cópia de trabalho de um repositório',

    'changes.navigation.emptyFolder': '{title} (vazia)',

    'changes.workspace.questionTitle': 'Pergunta',
    'changes.workspace.errorTitle': 'Erro',
    'changes.workspace.upgradeTitle': 'Atualização',
    'changes.workspace.alreadyOpened': '{path} já está aberto\nDeseja ir para essa visualização?',
    'changes.workspace.untitled': 'Sem título',
  },
  ja: {
    'changes.tree.all': 'すべて',
    'changes.ignore.extension': '*.{extension} を無視',
    'changes.ignore.extensionRecursively': '*.{extension} を再帰的に無視',
    'changes.ignore.name': '{name} を無視',
    'changes.ignore.nameRecursively': '{name} を再帰的に無視',

    'changes.list.failedToGetStatus': 'ステータスの取得に失敗しました：{error}',

    'changes.property.confirmDeletePrefix': 'プロパティを削除してもよろしいですか',
    'changes.property.confirmDeleteSuffix': '（{target} 内）？',
    'changes.property.confirmRevert': 'プロパティ {name} を元に戻しますか？',
    'changes.property.copyName': 'プロパティ名をコピー',
    'changes.property.copyOriginalValue': '元のプロパティ値をコピー',
    'changes.property.copyModifiedValue': '変更後のプロパティ値をコピー',
    'changes.error.unexpected': '予期しないエラーが発生しました：{error}',
    'changes.error.notUnderVersionControl': '「{path}」はバージョン管理下にありません。',
    'changes.error.emptyWorkingCopy': '作業コピーが空です。',

    'changes.history.failedToQueryRepository': '{path} のリポジトリ情報を取得できませんでした。',
    'changes.history.copyMessage': 'コミットメッセージをコピー',
    'changes.history.copyAuthor': '作成者をコピー',
    'changes.history.copyRevision': 'リビジョンをコピー',
    'changes.history.copyDate': '日付をコピー',
    'changes.history.copyUrl': 'URLをコピー',
    'changes.history.copyLocation': 'リポジトリ内のパスをコピー',
    'changes.history.mergeReset': 'マージ（リセット）',
    'changes.history.tabDetail': '詳細',
    'changes.history.tabChanges': '変更',
    'changes.history.tabSnapshot': 'スナップショット',
    'changes.history.saveSuccess': '{path} を保存しました',
    'changes.history.saveFailed': '{path} の保存に失敗しました：{error}',

    'changes.snapshot.saveSuccess': '{path} を保存しました',
    'changes.snapshot.saveFailed': '{path} の保存に失敗しました：{error}',

    'changes.remote.notWorkingCopyOfRepository': '{path} はリポジトリの作業コピーではありません。',

    'changes.navigation.emptyFolder': '{title}（空）',

    'changes.workspace.questionTitle': '確認',
    'changes.workspace.errorTitle': 'エラー',
    'changes.workspace.upgradeTitle': 'アップグレード',
    'changes.workspace.alreadyOpened':
      '{path} はすでに開かれています。\nこのビューに切り替えますか？',
    'changes.workspace.untitled': '無題',
  },
  de: {
    'changes.tree.all': 'Alle',
    'changes.ignore.extension': '*.{extension} ignorieren',
    'changes.ignore.extensionRecursively': '*.{extension} rekursiv ignorieren',
    'changes.ignore.name': '{name} ignorieren',
    'changes.ignore.nameRecursively': '{name} rekursiv ignorieren',

    'changes.list.failedToGetStatus': 'Status konnte nicht abgerufen werden: {error}',

    'changes.property.confirmDeletePrefix': 'Möchten Sie die Eigenschaft',
    'changes.property.confirmDeleteSuffix': ' in {target} löschen?',
    'changes.property.confirmRevert': 'Möchten Sie die Eigenschaft {name} rückgängig machen?',
    'changes.property.copyName': 'Eigenschaftsnamen kopieren',
    'changes.property.copyOriginalValue': 'Ursprünglichen Eigenschaftswert kopieren',
    'changes.property.copyModifiedValue': 'Geänderten Eigenschaftswert kopieren',
    'changes.error.unexpected': 'Unerwarteter Fehler: {error}',
    'changes.error.notUnderVersionControl': '„{path}“ steht nicht unter Versionskontrolle.',
    'changes.error.emptyWorkingCopy': 'Leere Arbeitskopie',

    'changes.history.failedToQueryRepository':
      'Das Repository für {path} konnte nicht abgefragt werden.',
    'changes.history.copyMessage': 'Protokollnachricht kopieren',
    'changes.history.copyAuthor': 'Autor kopieren',
    'changes.history.copyRevision': 'Revision kopieren',
    'changes.history.copyDate': 'Datum kopieren',
    'changes.history.copyUrl': 'URL kopieren',
    'changes.history.copyLocation': 'Repositoriumpfad kopieren',
    'changes.history.mergeReset': 'Zusammenführen (Zurücksetzen)',
    'changes.history.tabDetail': 'Details',
    'changes.history.tabChanges': 'Änderungen',
    'changes.history.tabSnapshot': 'Momentaufnahme',
    'changes.history.saveSuccess': '{path} wurde erfolgreich gespeichert.',
    'changes.history.saveFailed': '{path} konnte nicht gespeichert werden: {error}',

    'changes.snapshot.saveSuccess': '{path} wurde erfolgreich gespeichert.',
    'changes.snapshot.saveFailed': '{path} konnte nicht gespeichert werden: {error}',

    'changes.remote.notWorkingCopyOfRepository': '{path} ist keine Arbeitskopie eines Repositorys.',

    'changes.navigation.emptyFolder': '{title} (leer)',

    'changes.workspace.questionTitle': 'Frage',
    'changes.workspace.errorTitle': 'Fehler',
    'changes.workspace.upgradeTitle': 'Upgrade',
    'changes.workspace.alreadyOpened':
      '{path} ist bereits geöffnet.\nMöchten Sie zu dieser Ansicht wechseln?',
    'changes.workspace.untitled': 'Unbenannt',
  },
})
