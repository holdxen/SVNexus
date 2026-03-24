using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesTreeViewModel: ViewModelBase//, IRecipient<LocalTreeViewModel.OnLocalTreeItemChecked>, IRecipient<LocalTreeViewModel.OnLocalTreeItemExpanded>
{
    
    public partial class TreeItemViewModel: ViewModelBase
    {
        
        public ChangesTreeViewModel? Root { get; set; }
        
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }
        
        public ObservableCollection<TreeItemViewModel> Children {get; set;} = [];
    
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsReal))]
        [NotifyPropertyChangedFor(nameof(HasChild))]
        public required partial StatusEntry? StatusEntry { get; set; }
        
        public bool HasChild => StatusEntry?.NodeKind is NodeKind.Directory;


        public string PathSvgIcon => StatusEntry?.NodeKind.NodeKindIcon() ?? NodeKind.Directory.NodeKindIcon();


        public string Text
        {
            get => StatusEntry is not null ? System.IO.Path.GetFileName(StatusEntry.Path) : field;
            set;
        } = string.Empty;
        
        public string Path
        {
            get => StatusEntry is not null ? StatusEntry.Path : field;
            set;
        } = string.Empty;


        public bool IsDelete => StatusEntry?.NodeStatus is NodeStatus.Deleted or NodeStatus.Missing;


        public bool IsReal => StatusEntry is not null;
        
        public bool IsCheckable => StatusEntry is not null && StatusEntry.NodeStatus != NodeStatus.Normal;


        public string StatusToolTip => StatusEntry?.NodeStatus.ToString() ?? string.Empty;
    
    
        public string StatusSvgIcon => StatusEntry?.NodeStatus.NodeStatusIcon() ?? string.Empty;
    
        [ObservableProperty]
        public required partial string WorkingCopyPath { get; set; }
    
        [ObservableProperty]
        public required partial bool IsChecked { get; set; }


        public List<object> MenuItems { get; } = [];

        public List<object>? Buttons { get; } = [];
        
        
        // public required WeakReferenceMessenger Messenger { get; init; }


        partial void OnIsCheckedChanged(bool value)
        {
            // Messenger.Send(new OnLocalTreeItemChecked()
            // {
            //     IsChecked = value,
            //     ItemModel = this
            // });
            Root?.OnItemChecked(this, value);
        }

        partial void OnIsExpandedChanged(bool value)
        {
            Root?.OnItemExpanded(this, value);
        }
    }
    
    public ObservableCollection<TreeItemViewModel> Items { get; set; } = [];
    
    [ObservableProperty]
    public partial TreeItemViewModel? SelectedItem { get; set; }

    [ObservableProperty]
    public partial string WorkingCopyPath { get; set; } = string.Empty;

    public Dictionary<string, StatusEntry> CheckedItems { get; set; } = [];
    
    public List<string> ExpandedItems { get; set; } = [];

    [ObservableProperty]
    public partial bool? AllChecked { get; set; }
    
    private bool BlockSignal { get; set; }

    private int ItemCount { get; set; }

    [ObservableProperty]
    public partial bool SearchMode { get; set; }
    
    [ObservableProperty]
    public partial string SearchText { get; set; } = string.Empty;
    
    
    public List<TreeItemViewModel> DisplayItems { get; set; } = [];
    
    [ObservableProperty]
    public partial bool ShowRoot { get; set; }

    
    private TreeItemViewModel? _root;


    partial void OnShowRootChanged(bool value)
    {
        if (_root is null)
        {
            return;
        }

        Items.Clear();
        if (value)
        {
            Items.Add(_root);
            if (_root.StatusEntry is not null)
            {
                if (_root.IsChecked)
                {
                    CheckedItems[_root.StatusEntry.Path] = _root.StatusEntry;
                }

                ItemCount++;
            }
        }
        else
        {
            Items.AddRange(_root.Children);
            if (_root.StatusEntry is not null)
            {
                CheckedItems.Remove(_root.StatusEntry.Path);
                ItemCount--;
            }
        }
        UpdateAllChecked();
    }


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
    

    partial void OnAllCheckedChanged(bool? value)
    {
        if (BlockSignal)
        {
            return;
        }

        if (value is null)
        {
            return;
        }

        BlockSignal = true;
        foreach (var item in Items)
        {
            SetItemChecked(item, value.GetValueOrDefault());
        }
        BlockSignal = false;
    }

    private void SetItemChecked(TreeItemViewModel item, bool value)
    {
        if (item.IsReal)
        {
            item.IsChecked = value;
        }
        foreach (var child in item.Children)
        {
            SetItemChecked(child, value);
        }
    }

    private void UpdateAllChecked()
    {
        if (BlockSignal)
        {
            return;
        }
        BlockSignal = true;
        if (ItemCount == 0)
        {
            AllChecked = false;
        }
        else if (CheckedItems.Count == ItemCount)
        {
            AllChecked = true;
        }
        else if (CheckedItems.Count == 0)
        {
            AllChecked = false;
        }
        else
        {
            AllChecked = null;
        }
        BlockSignal = false;
    }
    
    

    partial void OnSelectedItemChanged(TreeItemViewModel? value)
    {
        Manager.Default.Send(new OnSelectedItemChanged(value?.StatusEntry), Token);
    }


    // public void Receive(OnLocalTreeItemChecked message)
    // {
    //     if (message.ItemModel.StatusEntry is null)
    //     {
    //         return;
    //     }
    //     if (message.IsChecked)
    //     {
    //         SelectedItems.Add(message.ItemModel.StatusEntry.Path);
    //     }
    //     else
    //     {
    //         SelectedItems.RemoveAll((e) => e == message.ItemModel.StatusEntry.Path);
    //     }
    // }

    public void OnItemChecked(TreeItemViewModel item, bool check)
    {
        if (item.StatusEntry is null)
        {
            return;
        }
        if (check)
        {
            CheckedItems.Add(item.StatusEntry.Path, item.StatusEntry);
        }
        else
        {
            // CheckedItems.RemoveAll((e) => e == item.StatusEntry.Path);
            CheckedItems.Remove(item.StatusEntry.Path);
        }
        
        UpdateAllChecked();
    }

    public void Update(StatusEntry[] statusEntries)
    {
        InvokeLoadedAction(() =>
        {
            Items.Clear();
            var root = new TreeItemViewModel
            {
                StatusEntry = null,
                WorkingCopyPath = WorkingCopyPath,
                Text = WorkingCopyPath.GetFileName(),
                IsExpanded = ExpandedItems.Contains(WorkingCopyPath),
                Path = WorkingCopyPath,
                IsChecked = false,
            };
        
            var cleanExpandedItems = new List<string>();
            var cleanCheckedItems = new Dictionary<string, StatusEntry>();
        
            if (root.IsExpanded)
            {
                cleanExpandedItems.Add(root.Path);    
            }

            foreach (var statusEntry in statusEntries)
            {
                if (statusEntry.Path == WorkingCopyPath)
                {
                    root.StatusEntry = statusEntry;
                    root.IsChecked = CheckedItems.ContainsKey(statusEntry.Path);
                    if (root.IsChecked)
                    {
                        cleanCheckedItems.Add(statusEntry.Path, statusEntry);
                    }

                    continue;
                }
                var path = statusEntry.Path.TrimStartString(WorkingCopyPath).TrimStartPathSeparatorChar();

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
                        var itemPath = $"{WorkingCopyPath}/{parentPath}/{part}";
                        var item = new TreeItemViewModel
                        {
                            WorkingCopyPath = WorkingCopyPath,
                            // Messenger = Messenger,
                            Text = part,
                            StatusEntry = null,
                            IsChecked = false,
                            Path = itemPath,
                            IsExpanded = ExpandedItems.Contains(itemPath),
                        };
                        if (item.IsExpanded)
                        {
                            cleanExpandedItems.Add(itemPath);
                        }
                        parentItem.Children.Add(item);
                        parentItem = item;
                        if (index == parts.Length - 1)
                        {
                            item.StatusEntry = statusEntry;
                            item.IsChecked = CheckedItems.ContainsKey(statusEntry.Path);
                            if (item.IsChecked)
                            {
                                cleanCheckedItems.Add(statusEntry.Path, statusEntry);
                            }

                        }

                        item.Root = this;
                    }
                    else
                    {
                        if (index == parts.Length - 1)
                        {
                            first.StatusEntry = statusEntry;
                            first.IsChecked = CheckedItems.ContainsKey(statusEntry.Path);
                            if (first.IsChecked)
                            {
                                cleanCheckedItems.Add(statusEntry.Path, statusEntry);
                            }
                            first.IsExpanded = ExpandedItems.Contains(statusEntry.Path);
                            break;
                        }

                        parentItem = first;
                    }

                    index++;
                    parentPath += "/" + part;
                }

            }

            root.Root = this;
            if (_root is null)
            {
                root.IsExpanded = true;
            }
            _root = root;
            
            
            OnShowRootChanged(ShowRoot);

            CheckedItems = cleanCheckedItems;
            ExpandedItems = cleanExpandedItems;

            ItemCount = statusEntries.Length;
        
            UpdateAllChecked();
        });

    }

    // public void Receive(OnLocalTreeItemExpanded message)
    // {
    //     if (message.IsExpanded)
    //     {
    //         ExpandedItems.Add(message.ItemModel.Path);
    //     }
    //     else
    //     {
    //         ExpandedItems.RemoveAll((e) => e == message.ItemModel.Path);
    //     }
    // }

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
}