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

    'svnDialogs.branchTag.description':
      '把「复制来源」处的目录在服务器上复制成一个新副本，作为分支或标签。创建时会直接提交到仓库，本地工作副本需要 update 之后才能看到它。',
    'svnDialogs.branchTag.source': '复制来源',
    'svnDialogs.branchTag.repositoryTree': '选择存放位置',
    'svnDialogs.branchTag.baseUrl': '存放位置的 URL',
    'svnDialogs.branchTag.name': '新分支/标签名称',
    'svnDialogs.branchTag.destination': '将创建的地址',
    'svnDialogs.branchTag.commitMessage': '提交信息',
    'svnDialogs.branchTag.baseRequired':
      '还没有确定存放位置：请在上面的目录树里选一个，或填写存放位置的 URL',
    'svnDialogs.branchTag.nameRequired': '还没有填写新分支/标签的名称',
    'svnDialogs.branchTag.notVersioned': '{path} 不是受版本控制的路径',
    'svnDialogs.branchTag.noReadableRoot': '读不到仓库里的任何层级，请手动填写存放位置的 URL',
    'svnDialogs.branchTag.created': '已创建（版本 {revision}）',
    'svnDialogs.branchTag.createdNoRevision': '已创建 {url}',
    'svnDialogs.branchTag.failed': '创建失败：{error}',
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

    'svnDialogs.branchTag.description':
      'Copies the folder under "Copy from" into a new copy on the server, as a branch or tag. Creating it commits straight to the repository, so your working copy needs an update before it shows up locally.',
    'svnDialogs.branchTag.source': 'Copy from',
    'svnDialogs.branchTag.repositoryTree': 'Choose where to put it',
    'svnDialogs.branchTag.baseUrl': 'Parent folder URL',
    'svnDialogs.branchTag.name': 'Branch/tag name',
    'svnDialogs.branchTag.destination': 'Will be created at',
    'svnDialogs.branchTag.commitMessage': 'Commit message',
    'svnDialogs.branchTag.baseRequired':
      'No destination yet: pick a folder in the tree above, or fill in the parent folder URL',
    'svnDialogs.branchTag.nameRequired': 'The branch/tag name is still empty',
    'svnDialogs.branchTag.notVersioned': '{path} is not a versioned path',
    'svnDialogs.branchTag.noReadableRoot':
      'Could not read any level of the repository; fill in the parent folder URL manually',
    'svnDialogs.branchTag.created': 'Created (revision {revision})',
    'svnDialogs.branchTag.createdNoRevision': 'Created {url}',
    'svnDialogs.branchTag.failed': 'Failed to create: {error}',
  },
  es: {
    'svnDialogs.add.failed': 'No se pudo agregar el archivo ({path}): {error}',

    'svnDialogs.checkout.progressTitle': 'Descargando...',
    'svnDialogs.checkout.descriptionUrl': 'URL',
    'svnDialogs.checkout.descriptionPath': 'Ruta',
    'svnDialogs.checkout.descriptionRevision': 'Revisión',
    'svnDialogs.checkout.current': 'Actual: {name}',
    'svnDialogs.checkout.percentage': 'Porcentaje: {percent}%',
    'svnDialogs.checkout.unknown': 'desconocido',
    'svnDialogs.checkout.failed': 'Falló la descarga: {error}',

    'svnDialogs.commit.select': 'Seleccionar',
    'svnDialogs.commit.actual': 'Real',
    'svnDialogs.commit.successAt': 'Confirmado correctamente (revisión {revision})',

    'svnDialogs.update.success': '{path} se actualizó correctamente',
    'svnDialogs.update.successAt': '{path} se actualizó correctamente a la r{revision}',
    'svnDialogs.update.failed': 'Falló la actualización: {error}',

    'svnDialogs.switch.success': 'Se cambió a la r{revision}',
    'svnDialogs.switch.failed': 'Falló el cambio: {error}',

    'svnDialogs.lock.action': 'Acción:',

    'svnDialogs.branchTag.description':
      'Copia la carpeta indicada en "Copiar desde" a una nueva copia en el servidor, como una rama o una etiqueta. Al crearla, se confirma directamente en el repositorio, así que tu copia de trabajo necesita una actualización antes de poder verla localmente.',
    'svnDialogs.branchTag.source': 'Copiar desde',
    'svnDialogs.branchTag.repositoryTree': 'Elige dónde ubicarla',
    'svnDialogs.branchTag.baseUrl': 'URL de la carpeta contenedora',
    'svnDialogs.branchTag.name': 'Nombre de la rama o etiqueta',
    'svnDialogs.branchTag.destination': 'Se creará en',
    'svnDialogs.branchTag.commitMessage': 'Mensaje de confirmación',
    'svnDialogs.branchTag.baseRequired':
      'Aún no hay destino: elige una carpeta en el árbol de arriba o completa la URL de la carpeta contenedora',
    'svnDialogs.branchTag.nameRequired': 'Todavía no escribiste el nombre de la rama o etiqueta',
    'svnDialogs.branchTag.notVersioned': '{path} no es una ruta versionada',
    'svnDialogs.branchTag.noReadableRoot':
      'No se pudo leer ningún nivel del repositorio; completa manualmente la URL de la carpeta contenedora',
    'svnDialogs.branchTag.created': 'Creada (revisión {revision})',
    'svnDialogs.branchTag.createdNoRevision': 'Creada en {url}',
    'svnDialogs.branchTag.failed': 'Falló la creación: {error}',
  },
  pt: {
    'svnDialogs.add.failed': 'Falha ao adicionar o arquivo ({path}): {error}',

    'svnDialogs.checkout.progressTitle': 'Fazendo checkout...',
    'svnDialogs.checkout.descriptionUrl': 'URL',
    'svnDialogs.checkout.descriptionPath': 'Caminho',
    'svnDialogs.checkout.descriptionRevision': 'Revisão',
    'svnDialogs.checkout.current': 'Atual: {name}',
    'svnDialogs.checkout.percentage': 'Porcentagem: {percent}%',
    'svnDialogs.checkout.unknown': 'desconhecido',
    'svnDialogs.checkout.failed': 'Falha ao fazer checkout: {error}',

    'svnDialogs.commit.select': 'Selecionar',
    'svnDialogs.commit.actual': 'Real',
    'svnDialogs.commit.successAt': 'Consolidado com sucesso (revisão {revision})',

    'svnDialogs.update.success': '{path} foi atualizado com sucesso',
    'svnDialogs.update.successAt': '{path} foi atualizado com sucesso para a r{revision}',
    'svnDialogs.update.failed': 'Falha ao atualizar: {error}',

    'svnDialogs.switch.success': 'Alternado para a r{revision}',
    'svnDialogs.switch.failed': 'Falha ao alternar: {error}',

    'svnDialogs.lock.action': 'Ação:',

    'svnDialogs.branchTag.description':
      'Copia a pasta indicada em "Copiar de" para uma nova cópia no servidor, como uma ramificação (branch) ou etiqueta. A criação é consolidada diretamente no repositório, então sua cópia de trabalho precisa de uma atualização antes de aparecer localmente.',
    'svnDialogs.branchTag.source': 'Copiar de',
    'svnDialogs.branchTag.repositoryTree': 'Escolha onde colocá-la',
    'svnDialogs.branchTag.baseUrl': 'URL da pasta pai',
    'svnDialogs.branchTag.name': 'Nome da ramificação (branch) ou etiqueta',
    'svnDialogs.branchTag.destination': 'Será criada em',
    'svnDialogs.branchTag.commitMessage': 'Mensagem de consolidação',
    'svnDialogs.branchTag.baseRequired':
      'Ainda não há destino: escolha uma pasta na árvore acima ou preencha a URL da pasta pai',
    'svnDialogs.branchTag.nameRequired':
      'O nome da ramificação (branch) ou etiqueta ainda está vazio',
    'svnDialogs.branchTag.notVersioned': '{path} não é um caminho versionado',
    'svnDialogs.branchTag.noReadableRoot':
      'Não foi possível ler nenhum nível do repositório; preencha manualmente a URL da pasta pai',
    'svnDialogs.branchTag.created': 'Criada (revisão {revision})',
    'svnDialogs.branchTag.createdNoRevision': 'Criada em {url}',
    'svnDialogs.branchTag.failed': 'Falha ao criar: {error}',
  },
  ja: {
    'svnDialogs.add.failed': 'ファイルの追加に失敗しました（{path}）：{error}',

    'svnDialogs.checkout.progressTitle': 'チェックアウト中…',
    'svnDialogs.checkout.descriptionUrl': 'URL',
    'svnDialogs.checkout.descriptionPath': 'パス',
    'svnDialogs.checkout.descriptionRevision': 'リビジョン',
    'svnDialogs.checkout.current': '現在：{name}',
    'svnDialogs.checkout.percentage': 'パーセント：{percent}%',
    'svnDialogs.checkout.unknown': '不明',
    'svnDialogs.checkout.failed': 'チェックアウトに失敗しました：{error}',

    'svnDialogs.commit.select': '選択',
    'svnDialogs.commit.actual': '実際',
    'svnDialogs.commit.successAt': 'コミットに成功しました（リビジョン {revision}）',

    'svnDialogs.update.success': '{path} の更新に成功しました。',
    'svnDialogs.update.successAt': '{path} を r{revision} に更新しました。',
    'svnDialogs.update.failed': '更新に失敗しました：{error}',

    'svnDialogs.switch.success': 'r{revision} に切り替えました。',
    'svnDialogs.switch.failed': '切り替えに失敗しました：{error}',

    'svnDialogs.lock.action': '操作：',

    'svnDialogs.branchTag.description':
      '「コピー元」のディレクトリを、ブランチまたはタグとしてサーバー上に新しいコピーとして複製します。作成するとリポジトリに直接コミットされるため、ローカルの作業コピーで確認するには更新が必要です。',
    'svnDialogs.branchTag.source': 'コピー元',
    'svnDialogs.branchTag.repositoryTree': '作成先を選択',
    'svnDialogs.branchTag.baseUrl': '作成先の URL',
    'svnDialogs.branchTag.name': '新しいブランチ / タグの名前',
    'svnDialogs.branchTag.destination': '作成される URL',
    'svnDialogs.branchTag.commitMessage': 'コミットメッセージ',
    'svnDialogs.branchTag.baseRequired':
      '作成先がまだ決まっていません：上のディレクトリツリーで選択するか、作成先の URL を入力してください。',
    'svnDialogs.branchTag.nameRequired': '新しいブランチ / タグの名前が入力されていません。',
    'svnDialogs.branchTag.notVersioned': '{path} はバージョン管理下のパスではありません。',
    'svnDialogs.branchTag.noReadableRoot':
      'リポジトリのどの階層も読み取れませんでした。作成先の URL を手動で入力してください。',
    'svnDialogs.branchTag.created': '作成しました（リビジョン {revision}）',
    'svnDialogs.branchTag.createdNoRevision': '{url} を作成しました',
    'svnDialogs.branchTag.failed': '作成に失敗しました：{error}',
  },
  de: {
    'svnDialogs.add.failed': 'Datei konnte nicht hinzugefügt werden ({path}): {error}',

    'svnDialogs.checkout.progressTitle': 'Auschecken...',
    'svnDialogs.checkout.descriptionUrl': 'URL',
    'svnDialogs.checkout.descriptionPath': 'Pfad',
    'svnDialogs.checkout.descriptionRevision': 'Revision',
    'svnDialogs.checkout.current': 'Aktuell: {name}',
    'svnDialogs.checkout.percentage': 'Prozent: {percent}%',
    'svnDialogs.checkout.unknown': 'unbekannt',
    'svnDialogs.checkout.failed': 'Auschecken fehlgeschlagen: {error}',

    'svnDialogs.commit.select': 'Auswählen',
    'svnDialogs.commit.actual': 'Tatsächlich',
    'svnDialogs.commit.successAt': 'Erfolgreich übernommen (Revision {revision})',

    'svnDialogs.update.success': '{path} wurde erfolgreich aktualisiert.',
    'svnDialogs.update.successAt': '{path} wurde auf r{revision} aktualisiert.',
    'svnDialogs.update.failed': 'Aktualisieren fehlgeschlagen: {error}',

    'svnDialogs.switch.success': 'Auf r{revision} gewechselt.',
    'svnDialogs.switch.failed': 'Wechseln fehlgeschlagen: {error}',

    'svnDialogs.lock.action': 'Aktion:',

    'svnDialogs.branchTag.description':
      'Kopiert das Verzeichnis unter „Kopieren von“ als neuen Zweig oder als neue Markierung auf dem Server. Beim Erstellen wird direkt in das Repository übernommen, daher muss die Arbeitskopie aktualisiert werden, bevor die Kopie lokal sichtbar ist.',
    'svnDialogs.branchTag.source': 'Kopieren von',
    'svnDialogs.branchTag.repositoryTree': 'Ablageort auswählen',
    'svnDialogs.branchTag.baseUrl': 'URL des übergeordneten Verzeichnisses',
    'svnDialogs.branchTag.name': 'Name des Zweigs oder der Markierung',
    'svnDialogs.branchTag.destination': 'Wird erstellt unter',
    'svnDialogs.branchTag.commitMessage': 'Protokollnachricht',
    'svnDialogs.branchTag.baseRequired':
      'Es ist noch kein Ziel festgelegt: Wählen Sie oben im Verzeichnisbaum ein Verzeichnis aus oder geben Sie die URL des übergeordneten Verzeichnisses ein.',
    'svnDialogs.branchTag.nameRequired':
      'Der Name des neuen Zweigs oder der neuen Markierung wurde noch nicht angegeben.',
    'svnDialogs.branchTag.notVersioned': '{path} ist kein versionierter Pfad.',
    'svnDialogs.branchTag.noReadableRoot':
      'Es konnte keine Ebene des Repositorys gelesen werden; geben Sie die URL des übergeordneten Verzeichnisses manuell ein.',
    'svnDialogs.branchTag.created': 'Erstellt (Revision {revision})',
    'svnDialogs.branchTag.createdNoRevision': 'Erstellt: {url}',
    'svnDialogs.branchTag.failed': 'Erstellen fehlgeschlagen: {error}',
  },
})
