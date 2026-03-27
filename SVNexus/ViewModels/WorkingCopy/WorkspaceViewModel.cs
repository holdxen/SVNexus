using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Security.Cryptography;
using System.Threading.Tasks;
using Avalonia.Media;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.ViewModels.WorkingCopy.Changes;
using Exception = SVNexus.Generated.Exception;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkspaceViewModel : ViewModelBase, IRecipient<WorkspaceViewModel.OnItemIsExpandedChanged>
{

    public class OnItemIsExpandedChanged()
    {
        public required TreeItemViewModel Item { get; set; }
        public required bool IsExpanded { get; set; }
    }

    public partial class TreeItemViewModel(ViewModelBase parent) : ViewModelBase(parent)
    {
        public StatusEntry? StatusEntry { get; set; }

        public string Name
        {
            get => StatusEntry is not null ? StatusEntry.Path.GetFileName() : field;
            set;
        } = string.Empty;

        public string AbsolutePath 
        { 
            get => StatusEntry is not null ? StatusEntry.Path : field;
            set;
        } = string.Empty;


        public string NodeKindIcon => StatusEntry?.NodeKind.NodeKindIcon() ?? string.Empty;
        
        public string NodeStatusIcon => StatusEntry?.NodeStatus.NodeStatusIcon() ?? string.Empty;

        public bool HasChild => StatusEntry?.NodeKind == NodeKind.Directory;
        
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }
        
        [ObservableProperty]
        public partial bool Ready { get; set; }
        
        public ObservableCollection<TreeItemViewModel> Children { get; set; } = [];

        partial void OnIsExpandedChanged(bool value)
        {
            if (Ready)
            {
                SendMessage(new OnItemIsExpandedChanged()
                {
                    IsExpanded = value,
                    Item = this
                });
            }
        }
    }

    private readonly Lazy<AsyncContext> _context;
    
    private readonly LimitedDictionary<string, WorkingCopyViewModel> _workingCopyViewModels = new()
    {
        Limit = 20
    };
    
    public string WorkspacePath { get; set; }
    
    public string? WorkspaceRoot { get; set; }
    
    
    [ObservableProperty]
    public partial TreeItemViewModel? SelectedTreeItem { get; set; }
    

    [ObservableProperty]
    public partial WorkingCopyViewModel? WorkingCopyViewModel { get; set; }
    
    public ObservableCollection<TreeItemViewModel> TreeItems { get; set; } = [];
    
    private List<string> _expandedItems = [];

    /// <inheritdoc/>
    public WorkspaceViewModel(string path, ViewModelBase parent) : base(parent)
    {
        WorkspacePath = path;

        _context = new Lazy<AsyncContext>(() =>
        {
            var hostId = SendMessage(new OnGetDialogHostId()); 
            var context = Engine.Engine.Instance.SimpleContext(hostId);
            return context;
        });
    }

    partial void OnSelectedTreeItemChanged(TreeItemViewModel? value)
    {
        if (value is null)
        {
            WorkingCopyViewModel = null;
            return;
        }


        if (_workingCopyViewModels.TryGetValue(value.Name, out var workingCopyViewModel))
        {
            WorkingCopyViewModel = workingCopyViewModel;
        }
        else
        {
            WorkingCopyViewModel = new WorkingCopyViewModel(this, value.AbsolutePath);
            
            _workingCopyViewModels.Add(value.AbsolutePath, WorkingCopyViewModel);
        }
        
        
        
    }


    private async Task RefreshTreeItems(bool initialize = false)
    {
        if (WorkspaceRoot is null)
        {
            return;
        }
        
        
        var statusOptions = new StatusOptions(WorkspaceRoot, new Revision.Working(), Depth.Infinity, true, false, false, false, false, false, null);

        var root = new TreeItemViewModel(this)
        {
            Name = WorkspaceRoot.GetFileName(),
            AbsolutePath = WorkspaceRoot,
        };


        List<string> newExpandedItems = [];
        
        TreeItemViewModel? selectedTreeItem = null;

        var receiver = new StatusReceiverDelegate()
        {
            OnStatusEntryAction = entry =>
            {
                try
                {
                    if (entry.NodeKind != NodeKind.Directory)
                    {
                        return;
                    }
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        var path = entry.Path.TrimStartString(WorkspaceRoot).TrimStartPathSeparatorChar();
                        if (string.IsNullOrEmpty(path))
                        {
                            root.StatusEntry = entry;
                            return;
                        }

                        var parts = path.Split('/');
                        var index = 0;
                        var parentItem = root;
                        var parentPath = string.Empty;
                        foreach (var part in parts)
                        {
                            var first = parentItem.Children.FirstOrDefault(e => e.Name == part);
                            if (first is null)
                            {
                                var itemPath = string.IsNullOrEmpty(parentPath) ? $"{WorkspaceRoot}/{part}" : $"{WorkspaceRoot}{parentPath}/{part}";
                                
                                var isExpanded = _expandedItems.Contains(itemPath);

                                if (initialize && !isExpanded && WorkspacePath.StartsWith(itemPath))
                                {
                                    isExpanded = true;
                                }
                                
                                var item = new TreeItemViewModel(this)
                                {
                                    Name = part,
                                    AbsolutePath = itemPath,
                                    IsExpanded = isExpanded,
                                };
                                if (initialize && itemPath == WorkspacePath)
                                {
                                    selectedTreeItem = item;
                                }
                                if (item.IsExpanded)
                                {
                                    newExpandedItems.Add(item.AbsolutePath);
                                }
                                
                                
                                
                                if (index == parts.Length - 1)
                                {
                                    item.StatusEntry = entry;
                                }
                                parentItem.Children.Add(item);
                                parentItem = item;
                            }
                            else
                            {
                                if (index == parts.Length - 1)
                                {
                                    first.StatusEntry = entry;
                                }

                                parentItem = first;
                            }
                            
                            index++;
                            parentPath += "/" + part;
                        }

                    });
                }
                catch (System.Exception e)
                {
                    Console.WriteLine(e);
                    throw;
                }
            }
        };

        await _context.Value.StatusNext(statusOptions, receiver);
        
        
        TreeItems.Clear();

        TreeItems.Add(root);

        if (initialize)
        {
            if (WorkspaceRoot == WorkspacePath)
            {
                SelectedTreeItem = root;
            }
            else if (selectedTreeItem is not null)
            {
                SelectedTreeItem = selectedTreeItem;
            }

            root.IsExpanded = true;
        }

        _expandedItems = newExpandedItems;
    }
    
    [RelayCommand]
    private async Task OnLoaded()
    {
        WorkspaceRoot = await _context.Value.GetWcRoot(WorkspacePath);
        await RefreshTreeItems(true);
    }

    public void Receive(OnItemIsExpandedChanged message)
    {
        if (message.IsExpanded)
        {
            _expandedItems.Add(message.Item.AbsolutePath);
        }
        else
        {
            _expandedItems.Remove(message.Item.AbsolutePath);
        }
    }
}