using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Utils;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesTreeViewModel : ViewModelBase, 
    IRecipient<ChangesTreeViewModel.OnItemIsExpandedChanged>,
    IRecipient<ChangesTreeViewModel.OnItemUpdated>
{

    public class OnItemIsExpandedChanged
    {
        public required TreeItemViewModel Item { get; init; }
        public required bool IsExpanded { get; init; }
    }

    public class OnItemUpdated;
    
    public partial class TreeItemViewModel(ViewModelBase parent): ViewModelBase(parent)
    {
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }
        
        public ObservableCollection<TreeItemViewModel> Children {get; set;} = [];
    
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsReal))]
        [NotifyPropertyChangedFor(nameof(HasChild))]
        [NotifyPropertyChangedFor(nameof(IsLocked))]
        [NotifyPropertyChangedFor(nameof(StatusIcon))]
        [NotifyPropertyChangedFor(nameof(Text))]
        [NotifyPropertyChangedFor(nameof(KindIcon))]
        [NotifyPropertyChangedFor(nameof(Path))]
        [NotifyPropertyChangedFor(nameof(IsDelete))]
        public required partial StatusEntry? StatusEntry { get; set; }
        
        
        public bool IsLocked => StatusEntry?.Lock is not null;
        
        public bool HasChild => StatusEntry?.NodeKind is NodeKind.Directory;


        public string KindIcon => (StatusEntry?.NodeKind ?? NodeKind.Directory).Icon();


        public string Text
        {
            get => StatusEntry is not null ? StatusEntry.Path.GetFileName() : field;
            set;
        } = string.Empty;
        
        public string Path
        {
            get => StatusEntry is not null ? StatusEntry.Path : field;
            set;
        } = string.Empty;


        public bool IsDelete => StatusEntry?.NodeStatus is WorkingCopyStatus.Deleted or WorkingCopyStatus.Missing;


        public bool IsReal => StatusEntry is not null;
        

        public string StatusToolTip => StatusEntry?.NodeStatus.ToString() ?? string.Empty;
    
    
        public string StatusIcon => StatusEntry?.NodeStatus.Icon() ?? string.Empty;
    
        [ObservableProperty]
        public required partial string WorkingCopyPath { get; set; }
    
        [ObservableProperty]
        public required partial bool IsChecked { get; set; }


        public List<object> MenuItems { get; } = [];

        public List<object>? Buttons { get; } = [];
        
        partial void OnIsExpandedChanged(bool value)
        {
            // Root?.OnItemExpanded(this, value);
            SendMessage(new OnItemIsExpandedChanged()
            {
                Item = this,
                IsExpanded = value
            });
        }

        [RelayCommand]
        private async Task OnLoaded()
        {
            if (StatusEntry is not null)
            {
                return;
            }

            var context = SendMessage(new OnGetContext()).Response;

            var statusOptions = new StatusOptions(Path, new Revision.Working(), Depth.Empty, true, false, false, false, false, false, null);

            var result = await context.Status(statusOptions);

            if (result.Entries.Length != 1)
            {
                Logger.Warn($"Unexpected entries length: {result.Entries.Length}");
                return;
            }
            
            StatusEntry = result.Entries[0];

            SendMessage(new OnItemUpdated());
        }
    }

    [ObservableProperty] public partial LoadingOrErrorState LoadingState { get; set; } = LoadingOrErrorState.MakeNone();
    
    // public ObservableCollection<TreeItemViewModel> Items { get; set; } = [];

    public ObservableCollection<TreeItemViewModel> Items
    {
        get
        {
            if (Root is null)
            {
                return [];
            }

            return ShowRoot ? [Root] : Root.Children;
        }
    }
    
    [ObservableProperty]
    public partial TreeItemViewModel? SelectedItem { get; set; }

    // [ObservableProperty]
    // public partial string WorkingCopyPath { get; set; } = string.Empty;

    // public Dictionary<string, StatusEntry> CheckedItems { get; set; } = [];
    
    public List<string> ExpandedItems { get; set; } = [];

    private int ItemCount { get; set; }

    [ObservableProperty]
    public partial bool SearchMode { get; set; }
    
    [ObservableProperty]
    public partial string SearchText { get; set; } = string.Empty;
    
    
    public List<TreeItemViewModel> DisplayItems { get; set; } = [];
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Items))]
    public partial bool ShowRoot { get; set; }


    [ObservableProperty]
    public partial TreeItemViewModel? Root { get; set; }

    /// <inheritdoc/>
    public ChangesTreeViewModel(ViewModelBase? parent = null) : base(parent)
    {
        SelectedItems?.CollectionChanged += (s, e) =>
        {
            NotifySelectedItemsChanged();
        };
    }

    [ObservableProperty]
    public partial ObservableCollection<TreeItemViewModel> SelectedItems { get; set; } = [];

    // partial void OnShowRootChanged(bool value)
    // {
    //     if (Root is null)
    //     {
    //         return;
    //     }
    //
    //     Items.Clear();
    //     if (value)
    //     {
    //         Items.Add(Root);
    //     }
    //     else
    //     {
    //         Items.AddRange(Root.Children);
    //     }
    // }


    [RelayCommand]
    private void Deselect()
    {
        SelectedItem = null;
    }

    [RelayCommand]
    private void ToggleSearchMode()
    {
        SearchMode = !SearchMode;
        
    }
    

    partial void OnSelectedItemsChanged(ObservableCollection<TreeItemViewModel> value)
    {
        NotifySelectedItemsChanged();
    }

    partial void OnSelectedItemChanged(TreeItemViewModel? value)
    {
        SendMessage(new Messages.OnSelectedItemChanged(value?.StatusEntry));
    }
    
    public void Update(StatusEntry[] statusEntries)
    {
        var workingCopyPath = SendMessage(new OnGetWorkingCopyPath()).Response;
        Items.Clear();
        var oldExpandedItems = ExpandedItems;
        // var oldCheckedItems = CheckedItems;

        ExpandedItems = [];
        // CheckedItems = [];
        
        var root = new TreeItemViewModel(this)
        {
            StatusEntry = null,
            WorkingCopyPath = workingCopyPath,
            Text = workingCopyPath.GetFileName(),
            IsExpanded = oldExpandedItems.Contains(workingCopyPath),
            Path = workingCopyPath,
            IsChecked = false,
        };
        
        // var cleanExpandedItems = new List<string>();
        // var cleanCheckedItems = new Dictionary<string, StatusEntry>();
        
        
        
        // if (root.IsExpanded)
        // {
        //     cleanExpandedItems.Add(root.Path);    
        // }

        foreach (var statusEntry in statusEntries)
        {
            if (statusEntry.Path == workingCopyPath)
            {
                root.StatusEntry = statusEntry;
                // root.IsChecked = oldCheckedItems.ContainsKey(statusEntry.Path);

                continue;
            }
            var path = statusEntry.Path.TrimStartString(workingCopyPath).TrimStartPathSeparatorChar();

            var parentItem = root;
            
            var parts = path.Split('/');

            var index = 0;
            var parentPath = string.Empty;
            
            foreach (var part in parts)
            {       
                var first = parentItem.Children.FirstOrDefault(e=> e.Text == part);
                if (first is null)
                {
                    // var itemPath = WorkingCopyPath + parentPath + "/" + part;
                    var itemPath = $"{workingCopyPath}/{parentPath}/{part}";
                    var item = new TreeItemViewModel(this)
                    {
                        WorkingCopyPath = workingCopyPath,
                        // Messenger = Messenger,
                        Text = part,
                        StatusEntry = null,
                        IsChecked = false,
                        Path = itemPath,
                        IsExpanded = oldExpandedItems.Contains(itemPath),
                    };
                    parentItem.Children.Add(item);
                    parentItem = item;
                    if (index == parts.Length - 1)
                    {
                        item.StatusEntry = statusEntry;
                        // item.IsChecked = oldCheckedItems.ContainsKey(statusEntry.Path);

                    }

                }
                else
                {
                    if (index == parts.Length - 1)
                    {
                        first.StatusEntry = statusEntry;
                        // first.IsChecked = oldCheckedItems.ContainsKey(statusEntry.Path);
                        first.IsExpanded = oldExpandedItems.Contains(statusEntry.Path);
                        break;
                    }

                    parentItem = first;
                }

                index++;
                parentPath += "/" + part;
            }

        }

        if (Root is null)
        {
            root.IsExpanded = true;
        }

        Root = root;
            
            
        // OnShowRootChanged(ShowRoot);

        ItemCount = statusEntries.Length;
        
    }


    public void OnItemExpanded(TreeItemViewModel item, bool expanded)
    {
        if (expanded)
        {
            ExpandedItems.Add(item.Path);
        }
        else
        {
            ExpandedItems.RemoveAll(e => e == item.Path);
        }
    }

    public void Receive(OnItemIsExpandedChanged message)
    {
        if (message.IsExpanded)
        {
            ExpandedItems.Add(message.Item.Path);
        }
        else
        {
            ExpandedItems.RemoveAll(e => e == message.Item.Path);
        }
    }

    public void NotifySelectedItemsChanged()
    {
        SendMessage(new Messages.OnSelectedItemsChanged(SelectedItems.Where(i => i.StatusEntry is not null).Select(i => i.StatusEntry!).ToList()));
    }

    public void Receive(OnItemUpdated message)
    {
        NotifySelectedItemsChanged();
    }
}