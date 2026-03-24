using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.History;

public partial class HistoryChangesViewModel : ViewModelBase
{

    public partial class ListViewModel : ViewModelBase
    {
        public ObservableCollection<ListItemViewModel> Items { get; } = [];
    }
    
    public partial class ListItemViewModel: ViewModelBase
    {

        [ObservableProperty]
        public partial string Path { get; set; } = string.Empty;


        public string FileName => Path == RelateToRoot ? "/" : Path.GetFileName();


        public string RelativeDirectory
        {
            get
            {
                var directory = Path.GetDirectoryName();
                if (directory == null)
                {
                    return string.Empty;
                }

                directory = directory.TrimStartString(RelateToRoot);
                
                return directory == "/" ? directory : directory.TrimStartPathSeparatorChar();
            }
        }

        public string RelativePath => Path == RelateToRoot ? "/" : Path.TrimStartString(RelateToRoot).TrimStartPathSeparatorChar();


        public string ActionIcon => Entry.Action.LogChangedPathActionIcon();


        public string ActionText => Entry.Action.ToString();
        
        public required LogChangedPathEntry Entry { get; set; }
        
        public required string RelateToRoot { get; set; }

        public string NodeKindIcon => Entry.NodeKind.NodeKindIcon();
    }
    
    public partial class TreeViewModel: ViewModelBase
    {
        
    }


    public const int ListViewIndex = 0;
    public const int TreeViewIndex = 1;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsTreeView))]
    [NotifyPropertyChangedFor(nameof(IsListView))]
    public partial int SelectedViewIndex { get; set; } = ListViewIndex;
    
    public bool IsTreeView => SelectedViewIndex == TreeViewIndex;
    
    public bool IsListView => SelectedViewIndex == ListViewIndex;

    public ListViewModel ChangedListViewModel { get; } = new();
    
    public TreeViewModel ChangedTreeViewViewModel { get; } = new();
    
    
    [ObservableProperty]
    public required partial Dictionary<string, LogChangedPathEntry> LogChangedPathEntries { get; set; }
    
    
    public required Revision CurrentRevision { get; set; }
    
    public required Revision CompareRevision { get; set; }
    
    public required string RelateToRoot { get; set; }

    public void Update()
    {
        ChangedListViewModel.Items.Clear();
        
        ChangedListViewModel.Items.AddRange(LogChangedPathEntries.Select(i => new ListItemViewModel()
        {
            Entry = i.Value,
            RelateToRoot = RelateToRoot,
            Path = i.Key
        }));
        
    }

    [RelayCommand]
    private void SwitchToListView()
    {
        SelectedViewIndex = ListViewIndex;
    }

    [RelayCommand]
    private void SwitchToTreeView()
    {
        SelectedViewIndex = TreeViewIndex;
    }
}