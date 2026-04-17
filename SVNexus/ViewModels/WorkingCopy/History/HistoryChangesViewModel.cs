using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Extension;
using SVNexus.Generated;
using SystemPath = System.IO.Path;

namespace SVNexus.ViewModels.WorkingCopy.History;

public partial class HistoryChangesViewModel : ViewModelLite
{

    public partial class ListViewModel : ViewModelBase
    {
        [ObservableProperty]
        public partial ObservableCollection<ListItemViewModel> Items { get; set; } = [];
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
                
                
                // if (Path == Current)
                // {
                //     return string.Empty;
                // }
                //
                // if (Path.StartsWith(Current))
                // {
                //     return Path.TrimStartString(Current).TrimStartPathSeparatorChar();
                // }
                //
                // return SystemPath.GetRelativePath(Current, Path.GetDirectoryName() ?? string.Empty);
                
                var directory = Path.GetDirectoryName();
                if (directory == null)
                {
                    return string.Empty;
                }

                if (!directory.StartsWith(RelateToRoot)) return SystemPath.GetRelativePath(RelateToRoot, directory);
                
                
                directory = directory.TrimStartString(RelateToRoot);
                
                return directory == "/" ? directory : directory.TrimStartPathSeparatorChar();

            }
        }

        public string RelativePath => Path == RelateToRoot ? "/" : Path.TrimStartString(RelateToRoot).TrimStartPathSeparatorChar();


        public string ActionIcon => Entry.Action.Icon();


        public string ActionText => Entry.Action.ToString();
        
        public required LogChangedPathEntry Entry { get; set; }
        
        public required string RelateToRoot { get; set; }

        public string NodeKindIcon => Entry.NodeKind.Icon();
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
    
    
    public required uint CurrentRevision { get; set; }
    
    public required uint? CompareRevision { get; set; }
    
    public required string RelateToRoot { get; set; }
    
    public required string CurrentPath { get; set; } = string.Empty;

    public void Update()
    {
        ChangedListViewModel.Items = new ObservableCollection<ListItemViewModel>(LogChangedPathEntries.Select(i => new ListItemViewModel()
        {
            Entry = i.Value,
            RelateToRoot = RelateToRoot,
            Path = i.Key,
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