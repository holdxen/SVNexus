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
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesTreeViewModel(ViewModelBase? parent = null): ViewModelBase(parent), 
    IRecipient<ChangesTreeViewModel.OnItemIsExpandedChanged>,
    IRecipient<ChangesTreeViewModel.OnItemIsCheckedChanged>
{

    public class OnItemIsExpandedChanged
    {
        public required TreeItemViewModel Item { get; init; }
        public required bool IsExpanded { get; init; }
    }

    public class OnItemIsCheckedChanged
    {
        public required TreeItemViewModel Item { get; init; }
        public required bool IsChecked { get; init; }
    }
    
    public partial class TreeItemViewModel(ViewModelBase parent): ViewModelBase(parent)
    {
        
        // public ChangesTreeViewModel? Root { get; set; }
        
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }
        
        public ObservableCollection<TreeItemViewModel> Children {get; set;} = [];
    
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsReal))]
        [NotifyPropertyChangedFor(nameof(HasChild))]
        public required partial StatusEntry? StatusEntry { get; set; }
        
        public bool HasChild => StatusEntry?.NodeKind is NodeKind.Directory;


        public string KindIcon => StatusEntry?.NodeKind.NodeKindIcon() ?? NodeKind.Directory.NodeKindIcon();


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
        
        public bool IsCheckable => StatusEntry is not null && StatusEntry.NodeStatus != WorkingCopyStatus.Normal;


        public string StatusToolTip => StatusEntry?.NodeStatus.ToString() ?? string.Empty;
    
    
        public string StatusIcon => StatusEntry?.NodeStatus.NodeStatusIcon() ?? string.Empty;
    
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
            // Root?.OnItemChecked(this, value);
            SendMessage(new OnItemIsCheckedChanged()
            {
                Item = this,
                IsChecked = value
            });
        }

        partial void OnIsExpandedChanged(bool value)
        {
            // Root?.OnItemExpanded(this, value);
            SendMessage(new OnItemIsExpandedChanged()
            {
                Item = this,
                IsExpanded = value
            });
        }
    }
    
    public ObservableCollection<TreeItemViewModel> Items { get; set; } = [];
    
    [ObservableProperty]
    public partial TreeItemViewModel? SelectedItem { get; set; }

    // [ObservableProperty]
    // public partial string WorkingCopyPath { get; set; } = string.Empty;

    public Dictionary<string, StatusEntry> CheckedItems { get; set; } = [];
    
    public List<string> ExpandedItems { get; set; } = [];

    private int ItemCount { get; set; }

    [ObservableProperty]
    public partial bool SearchMode { get; set; }
    
    [ObservableProperty]
    public partial string SearchText { get; set; } = string.Empty;
    
    
    public List<TreeItemViewModel> DisplayItems { get; set; } = [];
    
    [ObservableProperty]
    public partial bool ShowRoot { get; set; }

    
    private TreeItemViewModel? _root;

    // private readonly Services.TypeService _typeService;
    // private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    // public ChangesTreeViewModel(IServiceProvider serviceProvider)
    // {
    //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
    //     _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
    //     
    //     Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
    // }

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
    


    // private void SetItemChecked(TreeItemViewModel item, bool value)
    // {
    //     if (item.IsReal)
    //     {
    //         item.IsChecked = value;
    //     }
    //     foreach (var child in item.Children)
    //     {
    //         SetItemChecked(child, value);
    //     }
    // }

    

    partial void OnSelectedItemChanged(TreeItemViewModel? value)
    {
        // Manager.Default.Send(new OnSelectedItemChanged(value?.StatusEntry), _typeService.Get<ChangesViewModel>());
        SendMessage(new OnSelectedItemChanged(value?.StatusEntry));
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

    // public void OnItemChecked(TreeItemViewModel item, bool check)
    // {
    //     if (item.StatusEntry is null)
    //     {
    //         return;
    //     }
    //     if (check)
    //     {
    //         CheckedItems.Add(item.StatusEntry.Path, item.StatusEntry);
    //     }
    //     else
    //     {
    //         // CheckedItems.RemoveAll((e) => e == item.StatusEntry.Path);
    //         CheckedItems.Remove(item.StatusEntry.Path);
    //     }
    //     
    // }
    
    public void Update(StatusEntry[] statusEntries)
    {
        var workingCopyPath = SendMessage(new OnGetWorkingCopyPath()).Response;
        Items.Clear();
        var oldExpandedItems = ExpandedItems;
        var oldCheckedItems = CheckedItems;

        ExpandedItems = [];
        CheckedItems = [];
        
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
                root.IsChecked = oldCheckedItems.ContainsKey(statusEntry.Path);

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
                        item.IsChecked = oldCheckedItems.ContainsKey(statusEntry.Path);

                    }

                }
                else
                {
                    if (index == parts.Length - 1)
                    {
                        first.StatusEntry = statusEntry;
                        first.IsChecked = oldCheckedItems.ContainsKey(statusEntry.Path);
                        first.IsExpanded = oldExpandedItems.Contains(statusEntry.Path);
                        break;
                    }

                    parentItem = first;
                }

                index++;
                parentPath += "/" + part;
            }

        }

        if (_root is null)
        {
            root.IsExpanded = true;
        }
        _root = root;
            
            
        OnShowRootChanged(ShowRoot);

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

    public void Receive(OnItemIsCheckedChanged message)
    {
        if (message.Item.StatusEntry is null)
        {
            return;
        }
        if (message.IsChecked)
        {
            CheckedItems.Add(message.Item.StatusEntry.Path, message.Item.StatusEntry);
        }
        else
        {
            // CheckedItems.RemoveAll((e) => e == item.StatusEntry.Path);
            CheckedItems.Remove(message.Item.StatusEntry.Path);
        }
    }
}