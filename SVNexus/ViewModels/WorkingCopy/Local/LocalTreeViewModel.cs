using System.Collections.Generic;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalTreeViewModel: ViewModelBase
{
    
    
    public partial class TreeItemViewModel: ViewModelBase
    {
    
        [ObservableProperty]
        private bool _isExpanded;
    
        public ObservableCollection<TreeItemViewModel> Children {get; set;} = [];
    
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsReal))]
        public required partial StatusEntry? StatusEntry { get; set; }


        public string PathSvgIcon => "";


        public string Text
        {
            get => StatusEntry is not null ? StatusEntry.Path.TrimStartString(Path) : field;
            set;
        } = string.Empty;


        public bool IsDelete => StatusEntry?.NodeStatus is NodeStatus.Deleted or NodeStatus.Missing;


        public bool IsReal => StatusEntry is not null;


        public string StatusToolTip => StatusEntry?.NodeStatus.ToString() ?? string.Empty;
    
    
        public string StatusSvgIcon => StatusEntry?.NodeStatus.NodeStatusIcon() ?? string.Empty;
    
        [ObservableProperty]
        public required partial string Path { get; set; }
    
        [ObservableProperty]
        public required partial bool IsSelected { get; set; }


        public List<object> MenuItems { get; } = [];

        public List<object>? Buttons { get; } = [];

    }
    
    public ObservableCollection<TreeItemViewModel> Items { get; set; } = [];
    
    [ObservableProperty]
    public partial object? SelectedItem { get; set; }
    
    
    public required string WorkingCopyPath { get; set; }

    
}