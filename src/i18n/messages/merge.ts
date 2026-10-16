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
  es: {
    'merge.description':
      'Aplica los cambios de un repositorio a una copia de trabajo destino: primero elige el modo de fusión y luego indica el origen de los cambios y el destino de la fusión.',
    'merge.mode.title': 'Modo de fusión',
    'merge.mode.rangeHint':
      'Aplica al destino los cambios hechos en la rama de origen dentro de un rango de revisiones; es la forma más común de fusionar.',
    'merge.mode.treesHint':
      'Compara dos orígenes y aplica la diferencia al destino; se usa comúnmente para sincronizar ramas o para fusionar en sentido inverso.',
    'merge.source.title': 'Origen de la fusión',
    'merge.source.placeholder': 'URL del repositorio o ruta local',
    'merge.source.pegTitle':
      'Peg (opcional, solo se necesita si la ruta de origen cambió de nombre):',
    'merge.source.rangeHint':
      'Los cambios posteriores a la revisión inicial, hasta la revisión final inclusive, se fusionarán en el destino.',
    'merge.source.olderTreePlaceholder': 'URL o ruta del árbol más antiguo',
    'merge.source.newerTreePlaceholder': 'URL o ruta del árbol más reciente',
    'merge.source.source1Title': 'Source 1 (base de la comparación):',
    'merge.source.source2Title': 'Source 2 (destino de la comparación):',
    'merge.source.treesHint': 'La diferencia entre Source 1 y Source 2 se aplicará al destino.',
    'merge.target.title': 'Destino de la fusión',
    'merge.target.pathTitle': 'Target (copia de trabajo que recibe la fusión):',
    'merge.advanced.title': 'Opciones avanzadas',
    'merge.advanced.ignoreMergeInfoHint':
      'Ignorar el registro de lo que ya se fusionó, permitiendo fusionar de nuevo los mismos cambios',
    'merge.advanced.ignoreAncestryHint':
      'Ignorar la ascendencia y comparar solo por diferencias de contenido',
    'merge.advanced.forceDeleteHint':
      'Forzar la eliminación de archivos con modificaciones locales durante la fusión',
    'merge.advanced.recordOnlyHint':
      'Registrar solo la información de la fusión; no modificar el contenido de la copia de trabajo',
    'merge.advanced.dryRunHint':
      'Simulación: previsualizar los cambios que se producirían sin aplicarlos',
    'merge.advanced.allowMixedRevisionHint':
      'Permitir la operación cuando la copia de trabajo está en revisiones mixtas',
  },
  pt: {
    'merge.description':
      'Aplica as alterações de um repositório a uma cópia de trabalho de destino: primeiro escolha o modo de mesclagem e depois informe a origem das alterações e o destino da mesclagem.',
    'merge.mode.title': 'Modo de mesclagem',
    'merge.mode.rangeHint':
      'Aplica ao destino as alterações feitas na ramificação de origem dentro de um intervalo de revisões; é a forma mais comum de mesclar.',
    'merge.mode.treesHint':
      'Compara duas origens e aplica a diferença ao destino; é usado comumente para sincronizar ramificações ou mesclar no sentido inverso.',
    'merge.source.title': 'Origem da mesclagem',
    'merge.source.placeholder': 'URL do repositório ou caminho local',
    'merge.source.pegTitle':
      'Peg (opcional, só é necessário se o caminho de origem foi renomeado):',
    'merge.source.rangeHint':
      'As alterações feitas após a revisão inicial, até a revisão final inclusive, serão mescladas no destino.',
    'merge.source.olderTreePlaceholder': 'URL ou caminho da árvore mais antiga',
    'merge.source.newerTreePlaceholder': 'URL ou caminho da árvore mais recente',
    'merge.source.source1Title': 'Source 1 (base da comparação):',
    'merge.source.source2Title': 'Source 2 (destino da comparação):',
    'merge.source.treesHint': 'A diferença entre Source 1 e Source 2 será aplicada ao destino.',
    'merge.target.title': 'Destino da mesclagem',
    'merge.target.pathTitle': 'Target (cópia de trabalho que recebe a mesclagem):',
    'merge.advanced.title': 'Opções avançadas',
    'merge.advanced.ignoreMergeInfoHint':
      'Ignorar o registro do que já foi mesclado, permitindo mesclar as mesmas alterações novamente',
    'merge.advanced.ignoreAncestryHint':
      'Ignorar a ancestralidade e comparar apenas pelas diferenças de conteúdo',
    'merge.advanced.forceDeleteHint':
      'Forçar a exclusão de arquivos com modificações locais durante a mesclagem',
    'merge.advanced.recordOnlyHint':
      'Registrar apenas as informações da mesclagem; não modificar o conteúdo da cópia de trabalho',
    'merge.advanced.dryRunHint': 'Simulação: pré-visualizar as alterações sem aplicá-las',
    'merge.advanced.allowMixedRevisionHint':
      'Permitir a operação quando a cópia de trabalho está em revisões mistas',
  },
  ja: {
    'merge.description':
      'リポジトリの変更を対象の作業コピーに適用します。最初にマージ方法を選択し、次に変更元とマージ対象を指定します。',
    'merge.mode.title': 'マージ方法',
    'merge.mode.rangeHint':
      '指定したリビジョン範囲でソースブランチに行われた変更を対象に適用します。最も一般的なマージ方法です。',
    'merge.mode.treesHint':
      '2 つのソースを比較し、その差分を対象に適用します。ブランチの同期や逆方向のマージによく使われます。',
    'merge.source.title': 'マージ元',
    'merge.source.placeholder': 'リポジトリ URL またはローカルパス',
    'merge.source.pegTitle': 'Peg（任意、ソースパスが名前変更された場合のみ必要）：',
    'merge.source.rangeHint':
      '開始リビジョンより後、終了リビジョンまでの変更が対象にマージされます。',
    'merge.source.olderTreePlaceholder': '古いツリーの URL またはパス',
    'merge.source.newerTreePlaceholder': '新しいツリーの URL またはパス',
    'merge.source.source1Title': 'Source 1（比較の基準）：',
    'merge.source.source2Title': 'Source 2（比較の対象）：',
    'merge.source.treesHint': 'Source 1 と Source 2 の差分が対象に適用されます。',
    'merge.target.title': 'マージ対象',
    'merge.target.pathTitle': 'Target（マージを受け取る作業コピー）：',
    'merge.advanced.title': '詳細オプション',
    'merge.advanced.ignoreMergeInfoHint':
      'マージ済みの記録を無視し、同じ変更を再度マージできるようにします。',
    'merge.advanced.ignoreAncestryHint': '系譜を無視し、内容の差分のみで比較します。',
    'merge.advanced.forceDeleteHint':
      'マージ時にローカルの変更があるファイルを強制的に削除します。',
    'merge.advanced.recordOnlyHint': 'マージ情報のみを記録し、作業コピーの内容は変更しません。',
    'merge.advanced.dryRunHint': 'ドライラン：実際に変更せず、生成される変更をプレビューします。',
    'merge.advanced.allowMixedRevisionHint':
      '作業コピーが混在リビジョンの状態でも操作を許可します。',
  },
  de: {
    'merge.description':
      'Wenden Sie Änderungen aus einem Repository auf eine Arbeitskopie als Ziel an: Wählen Sie zuerst den Zusammenführungsmodus und geben Sie dann die Quelle der Änderungen und das Zusammenführungsziel an.',
    'merge.mode.title': 'Zusammenführungsmodus',
    'merge.mode.rangeHint':
      'Wendet die im angegebenen Revisionsbereich auf dem Quellzweig vorgenommenen Änderungen auf das Ziel an; dies ist die gebräuchlichste Form des Zusammenführens.',
    'merge.mode.treesHint':
      'Vergleicht zwei Quellen und wendet die Unterschiede auf das Ziel an; wird häufig zum Synchronisieren von Zweigen oder für umgekehrtes Zusammenführen verwendet.',
    'merge.source.title': 'Zusammenführungsquelle',
    'merge.source.placeholder': 'Repository-URL oder lokaler Pfad',
    'merge.source.pegTitle':
      'Peg (optional, nur erforderlich, wenn der Quellpfad umbenannt wurde):',
    'merge.source.rangeHint':
      'Änderungen nach der Startrevision bis einschließlich der Endrevision werden in das Ziel zusammengeführt.',
    'merge.source.olderTreePlaceholder': 'URL oder Pfad des älteren Baums',
    'merge.source.newerTreePlaceholder': 'URL oder Pfad des neueren Baums',
    'merge.source.source1Title': 'Source 1 (Vergleichsbasis):',
    'merge.source.source2Title': 'Source 2 (Vergleichsziel):',
    'merge.source.treesHint':
      'Die Unterschiede zwischen Source 1 und Source 2 werden auf das Ziel angewendet.',
    'merge.target.title': 'Zusammenführungsziel',
    'merge.target.pathTitle': 'Target (Arbeitskopie, die die Zusammenführung erhält):',
    'merge.advanced.title': 'Erweiterte Optionen',
    'merge.advanced.ignoreMergeInfoHint':
      'Ignoriert die Aufzeichnung bereits zusammengeführter Änderungen, sodass dieselben Änderungen erneut zusammengeführt werden können',
    'merge.advanced.ignoreAncestryHint':
      'Ignoriert die Abstammung und vergleicht nur anhand der Inhaltsunterschiede',
    'merge.advanced.forceDeleteHint':
      'Erzwingt beim Zusammenführen das Löschen von Dateien mit lokalen Änderungen',
    'merge.advanced.recordOnlyHint':
      'Zeichnet nur die Zusammenführungsinformationen auf; der Inhalt der Arbeitskopie wird nicht geändert',
    'merge.advanced.dryRunHint':
      'Testlauf: Zeigt eine Vorschau der Änderungen an, ohne sie durchzuführen',
    'merge.advanced.allowMixedRevisionHint':
      'Erlaubt den Vorgang, wenn die Arbeitskopie gemischte Revisionen aufweist.',
  },
})
