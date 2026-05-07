using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Utils;
using SystemPath = System.IO.Path;

namespace SVNexus.ViewModels.WorkingCopy.History;

public partial class HistoryChangesViewModel : ViewModelBase
{
    
    public partial class ListItemViewModel: ViewModelBase
    {
        [ObservableProperty] public partial bool IsVisible { get; set; } = true;
        
        [ObservableProperty]
        public partial string Path { get; set; } = string.Empty;


        public string FileName => Path == RelateToRoot ? "/" : Path.GetFileName();


        public string RelativeDirectory
        {
            get
            {
                if (RelateToRoot == Path)
                {
                    return string.Empty;
                }

                var relate = SystemPath.GetRelativePath(RelateToRoot, Path);
                
                return relate.GetDirectoryName() ?? string.Empty;
            }
        }

        public string RelativePath => Path == RelateToRoot ? "/" : Path.TrimStartString(RelateToRoot).TrimStartPathSeparatorChar();


        public string ActionIcon => Entry.Action.Icon();


        public string ActionText => Entry.Action.ToString();
        
        public required LogChangedPathEntry Entry { get; set; }
        
        public required string RelateToRoot { get; set; }

        public string NodeKindIcon => Entry.NodeKind.Icon();
    }

    
    public required string RootUrl { get; set; }

    private readonly LimitedDictionary<string, DifferenceViewModel> _differenceViewModels = new()
    {
        Limit = 20
    };

    [ObservableProperty] 
    public partial int SelectedChangedItemIndex { get; set; } = -1;

    [ObservableProperty]
    public partial DifferenceViewModel DifferenceViewModel { get; set; }

    [ObservableProperty]
    public partial ObservableCollection<ListItemViewModel> ChangedItems { get; set; } = [];
    
    [ObservableProperty]
    public partial bool ShowChildrenOnly { get; set; }
    
    [ObservableProperty]
    public required partial Dictionary<string, LogChangedPathEntry> LogChangedPathEntries { get; set; }
    
    
    public required uint CurrentRevision { get; set; }
    
    public required uint? CompareRevision { get; set; }
    
    public required string RelateToRoot { get; set; }

    [ObservableProperty]
    public partial GridLength LeftPartWidth { get; set; } = new(1, GridUnitType.Star);

    
    public GridLength LeftPartRealWidth => _leftPartWidthValid ? new GridLength(LeftPartWidth.Value, GridUnitType.Pixel) : LeftPartWidth;

    private bool _leftPartWidthValid;

    /// <inheritdoc/>
    public HistoryChangesViewModel(ViewModelBase parent) : base(parent)
    {
        DifferenceViewModel = new DifferenceViewModel(this);
    }
    

    partial void OnSelectedChangedItemIndexChanged(int value)
    {
        if (value < 0 || value >= ChangedItems.Count) return;
        var change = ChangedItems[value];
        if (_differenceViewModels.TryGetValue(change.Path, out var differenceViewModel))
        {
            DifferenceViewModel = differenceViewModel;
        }
        else
        {
            DifferenceViewModel = new DifferenceViewModel(this);
            _differenceViewModels.Add(change.Path, DifferenceViewModel);
            Dispatcher.UIThread.InvokeAsync(async () =>
            {
                var current = change.Entry.Action == LogChangedPathAction.Delete
                    ? null
                    : new Revision.Number(CurrentRevision);
                var compared = change.Entry.Action == LogChangedPathAction.Add ? null : CompareRevision?.Map(e => new Revision.Number(CurrentRevision));
                if (ChangedItems[value].Entry.NodeKind == NodeKind.Directory)
                {
                    await DifferenceViewModel.CompareProperty(RootUrl + change.Path, new Revision.Number(CurrentRevision), compared, current);
                }
                else
                {
                    await DifferenceViewModel.Compare(RootUrl + change.Path, new Revision.Number(CurrentRevision), compared, current);
                }
            });
        }
    }
    
    partial void OnLeftPartWidthChanged(GridLength value)
    {
        _leftPartWidthValid = true;
    }

    partial void OnShowChildrenOnlyChanged(bool value)
    {
        if (value)
        {
            foreach (var item in ChangedItems)
            {
                item.IsVisible = item.Path.StartsWith(item.RelateToRoot);
            }
        }
        else
        {
            foreach (var item in ChangedItems)
            {
                item.IsVisible = true;
            }
        }
    }

    public void Update()
    {
        ChangedItems = new ObservableCollection<ListItemViewModel>(LogChangedPathEntries.Select(i => new ListItemViewModel()
        {
            Entry = i.Value,
            RelateToRoot = RelateToRoot,
            Path = i.Key,
        }));
        
    }

    // [RelayCommand]
    // private void SwitchToListView()
    // {
    //     SelectedViewIndex = ListViewIndex;
    // }
    //
    // [RelayCommand]
    // private void SwitchToTreeView()
    // {
    //     SelectedViewIndex = TreeViewIndex;
    // }
}