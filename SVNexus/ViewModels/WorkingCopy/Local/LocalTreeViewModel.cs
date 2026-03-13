using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalTreeViewModel: ViewModelBase//, IRecipient<LocalTreeViewModel.OnLocalTreeItemChecked>, IRecipient<LocalTreeViewModel.OnLocalTreeItemExpanded>
{
    
    public partial class TreeItemViewModel: ViewModelBase
    {
        
        public LocalTreeViewModel? Root { get; set; }

        [ObservableProperty]
        public partial bool IsExpanded { get; set; }
        
        public ObservableCollection<TreeItemViewModel> Children {get; set;} = [];
    
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsReal))]
        public required partial StatusEntry? StatusEntry { get; set; }


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
    
    // public class OnLocalTreeItemChecked
    // {
    //     public required bool IsChecked { get; init; }
    //     public required TreeItemViewModel ItemModel { get; init; }
    // }
    //
    // public class OnLocalTreeItemExpanded
    // {
    //     public required bool IsExpanded { get; init; }
    //     public required TreeItemViewModel ItemModel { get; init; }
    // }

    public override bool KeepAlive { get; set; } = true;


    public ObservableCollection<TreeItemViewModel> Items { get; set; } = [];
    
    [ObservableProperty]
    public partial TreeItemViewModel? SelectedItem { get; set; }

    [ObservableProperty]
    public partial string WorkingCopyPath { get; set; } = string.Empty;

    partial void OnWorkingCopyPathChanged(string value)
    {
        Console.WriteLine("WorkingCopyPath={0}", value);
    }


    public Dictionary<string, StatusEntry> CheckedItems { get; set; } = [];
    
    public List<string> ExpandedItems { get; set; } = [];

    [ObservableProperty]
    public partial bool? AllChecked { get; set; }
    
    private bool BlockSignal { get; set; }

    private int ItemCount { get; set; }

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
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            await InvokeLoadedAction(async () =>
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
        
        
        
        
                Console.WriteLine("Root.Text={0}, WorkingCopyPath={1}", root.Text, WorkingCopyPath);



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
                Items.AddRange(root.Children);

                CheckedItems = cleanCheckedItems;
                ExpandedItems = cleanExpandedItems;

                ItemCount = statusEntries.Length;
        
                UpdateAllChecked();
            });
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