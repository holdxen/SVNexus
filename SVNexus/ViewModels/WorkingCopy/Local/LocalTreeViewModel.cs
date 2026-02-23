using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.IO;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalTreeViewModel: ViewModelBase, IRecipient<LocalTreeViewModel.OnLocalTreeItemSelected>, IRecipient<LocalTreeViewModel.OnLocalTreeItemExpanded>
{
    
    
    public partial class TreeItemViewModel: ViewModelBase
    {
    
        [ObservableProperty]
        private bool _isExpanded;
    
        public ObservableCollection<TreeItemViewModel> Children {get; set;} = [];
    
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsReal))]
        public required partial StatusEntry? StatusEntry { get; set; }


        public string PathSvgIcon => StatusEntry?.NodeKind.NodeKindIcon() ?? NodeKind.Directory.NodeKindIcon();


        public string Text
        {
            get => StatusEntry is not null ? StatusEntry.Path.TrimStartString(WorkingCopyPath) : field;
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
        public required partial bool IsSelected { get; set; }


        public List<object> MenuItems { get; } = [];

        public List<object>? Buttons { get; } = [];
        
        
        public required WeakReferenceMessenger Messenger { get; init; }


        partial void OnIsSelectedChanged(bool value)
        {
            Messenger.Send(new OnLocalTreeItemSelected()
            {
                IsSelected = value,
                ItemModel = this
            });
        }

        partial void OnIsExpandedChanged(bool value)
        {
            Messenger.Send(new OnLocalTreeItemExpanded()
            {
                IsExpanded = value,
                ItemModel = this
            });
        }
    }
    
    public class OnLocalTreeItemSelected
    {
        public required bool IsSelected { get; init; }
        public required TreeItemViewModel ItemModel { get; init; }
    }
    
    public class OnLocalTreeItemExpanded
    {
        public required bool IsExpanded { get; init; }
        public required TreeItemViewModel ItemModel { get; init; }
    }

    public override bool KeepAlive { get; set; } = true;


    public ObservableCollection<TreeItemViewModel> Items { get; set; } = [];
    
    [ObservableProperty]
    public partial object? SelectedItem { get; set; }
    
    public required string WorkingCopyPath { get; set; }


    public List<string> SelectedItems { get; set; } = [];
    
    public List<string> ExpandedItems { get; set; } = [];


    public WeakReferenceMessenger Messenger { get; } = new();

    public LocalTreeViewModel()
    {
        Messenger.Register<OnLocalTreeItemSelected>(this);
        Messenger.Register<OnLocalTreeItemExpanded>(this);
    }


    public void Receive(OnLocalTreeItemSelected message)
    {
        if (message.ItemModel.StatusEntry is null)
        {
            return;
        }
        if (message.IsSelected)
        {
            SelectedItems.Add(message.ItemModel.StatusEntry.Path);
        }
        else
        {
            SelectedItems.RemoveAll((e) => e == message.ItemModel.StatusEntry.Path);
        }
    }

    public void Update(StatusEntry[] statusEntries)
    {
        Items.Clear();

        var root = new TreeItemViewModel
        {
            StatusEntry = null,
            WorkingCopyPath = WorkingCopyPath,
            IsSelected = false,
            Messenger = Messenger,
            Text = Path.GetFileName(WorkingCopyPath.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar)),
            IsExpanded = ExpandedItems.Contains(WorkingCopyPath),
            Path = WorkingCopyPath,
        };
        
        
        
        
        Console.WriteLine("Root.Text={0}, WorkingCopyPath={1}", root.Text, WorkingCopyPath);



        var cleanExpandedItems = new List<string>();
        var cleanSelectedItems = new List<string>();
        
        if (root.IsExpanded)
        {
            cleanExpandedItems.Add(root.Path);    
        }

        foreach (var statusEntry in statusEntries)
        {
            if (statusEntry.Path == WorkingCopyPath)
            {
                root.StatusEntry = statusEntry;
                root.IsSelected = SelectedItems.Contains(statusEntry.Path);
                if (root.IsSelected)
                {
                    cleanSelectedItems.Add(statusEntry.Path);
                }

                continue;
            }
            var path = statusEntry.Path.TrimStartString(WorkingCopyPath);

            var parentItem = root;
            
            var parts = path.Split('/');

            var index = 0;
            var parentPath = string.Empty;
            
            foreach (var part in parts)
            {       
                var first = parentItem.Children.FirstOrDefault(e=> e.Text == part);
                if (first is null)
                {
                    var itemPath = WorkingCopyPath + parentPath + "/" + part;
                    var item = new TreeItemViewModel
                    {
                        WorkingCopyPath = WorkingCopyPath,
                        Messenger = Messenger,
                        Text = part,
                        StatusEntry = null,
                        IsSelected = false,
                        Path = itemPath,
                        IsExpanded = ExpandedItems.Contains(itemPath)
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
                        item.IsSelected = SelectedItems.Contains(statusEntry.Path);
                        if (item.IsSelected)
                        {
                            cleanSelectedItems.Add(statusEntry.Path);
                        }

                    }
                }
                else
                {
                    if (index == parts.Length - 1)
                    {
                        first.StatusEntry = statusEntry;
                        first.IsSelected = SelectedItems.Contains(statusEntry.Path);
                        if (first.IsSelected)
                        {
                            cleanSelectedItems.Add(statusEntry.Path);
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

        Items.Add(root);

        SelectedItems = cleanSelectedItems;
        ExpandedItems = cleanExpandedItems;
    }

    public void Receive(OnLocalTreeItemExpanded message)
    {
        if (message.IsExpanded)
        {
            ExpandedItems.Add(message.ItemModel.Path);
        }
        else
        {
            ExpandedItems.RemoveAll((e) => e == message.ItemModel.Path);
        }
    }
}