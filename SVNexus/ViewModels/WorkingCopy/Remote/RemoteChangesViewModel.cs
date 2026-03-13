using System.Collections.Generic;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.Remote;

public partial class RemoteChangesViewModel : ViewModelBase
{
    public partial class ListViewModel: ViewModelBase
    {
        
    }
    
    public partial class TreeViewModel: ViewModelBase
    {
        
    }
    
    
    public const int ListViewIndex = 0;
    public const int TreeViewIndex = 1;

    [ObservableProperty]
    public partial int SelectedViewIndex { get; set; } = ListViewIndex;
    
    public bool IsTreeView => SelectedViewIndex == TreeViewIndex;
    
    public bool IsListView => SelectedViewIndex == ListViewIndex;
    
    
    public required Dictionary<string, LogChangedPathEntry> LogChangedPathEntries { get; set; } = [];
    
    
    public required Revision CurrentRevision { get; set; }
    
    public required Revision CompareRevision { get; set; }


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