using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkspaceViewModel : ViewModelBase, 
    IRecipient<WorkspaceViewModel.OnItemIsExpandedChanged>,
    IRecipient<OnGetContext>,
    IRecipient<OnGetWorkingCopyRoot>
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
        
        public ObservableCollection<TreeItemViewModel> Children { get; set; } = [];

        partial void OnIsExpandedChanged(bool value)
        {
            SendMessage(new OnItemIsExpandedChanged()
            {
                IsExpanded = value,
                Item = this
            });
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
    public partial ObservableCollection<TreeItemViewModel> SelectedTreeItems { get; set; } = [];
    
    

    [ObservableProperty]
    public partial WorkingCopyViewModel? WorkingCopyViewModel { get; set; }
    
    public ObservableCollection<TreeItemViewModel> TreeItems { get; set; } = [];
    
    private List<string> _expandedItems = [];

    public bool IsAddButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry is not null && i.StatusEntry?.NodeStatus == NodeStatus.Unversioned);
    
    public bool IsUpdateButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;

    public bool IsRefreshButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsRevertButtonEnable =>  SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsDiffButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeKind == NodeKind.Directory && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsPatchButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeKind == NodeKind.Directory && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsDeleteButtonEnable => SelectedTreeItems.Count == 1 && 
                                        SelectedTreeItems.First().StatusEntry?.NodeStatus is not (NodeStatus.Unversioned or NodeStatus.Added) &&
                                        SelectedTreeItems.First().AbsolutePath != WorkspaceRoot;
    
    public bool IsMkdirButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsCommitButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.Any(i => i.StatusEntry?.NodeStatus != NodeStatus.Unversioned);
    
    public bool IsSwitchButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsMergeButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    
    public bool IsRelocateButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().AbsolutePath == WorkspaceRoot;
    

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

        // Don't know why there is a warning about the reference may be null
        SelectedTreeItems?.CollectionChanged += SelectTreeItemsCollectionChanged;
    }


    private void NotifyButtonState()
    {
        OnPropertyChanged(nameof(IsAddButtonEnable));
        OnPropertyChanged(nameof(IsUpdateButtonEnable));
        OnPropertyChanged(nameof(IsRefreshButtonEnable));
        OnPropertyChanged(nameof(IsRevertButtonEnable));
        OnPropertyChanged(nameof(IsDiffButtonEnable));
        OnPropertyChanged(nameof(IsPatchButtonEnable));
        OnPropertyChanged(nameof(IsDeleteButtonEnable));
        OnPropertyChanged(nameof(IsMkdirButtonEnable));
        OnPropertyChanged(nameof(IsCommitButtonEnable));
        OnPropertyChanged(nameof(IsSwitchButtonEnable));
        OnPropertyChanged(nameof(IsMergeButtonEnable));
        OnPropertyChanged(nameof(IsRelocateButtonEnable));
    }

    partial void OnSelectedTreeItemsChanged(ObservableCollection<TreeItemViewModel> oldValue, ObservableCollection<TreeItemViewModel> newValue)
    {
        oldValue.CollectionChanged -= SelectTreeItemsCollectionChanged;
        newValue.CollectionChanged += SelectTreeItemsCollectionChanged;
        NotifyButtonState();
    }

    private void SelectTreeItemsCollectionChanged(object? sender, NotifyCollectionChangedEventArgs e)
    {
        NotifyButtonState();
    }

    partial void OnSelectedTreeItemChanged(TreeItemViewModel? value)
    {
        if (value is null || value.StatusEntry?.NodeStatus == NodeStatus.Unversioned)
        {
            WorkingCopyViewModel = null;
            return;
        }


        if (_workingCopyViewModels.TryGetValue(value.AbsolutePath, out var workingCopyViewModel))
        {
            if (WorkingCopyViewModel != workingCopyViewModel)
            {
                WorkingCopyViewModel = workingCopyViewModel;
            }
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

        var oldExpandedItems = _expandedItems;
        _expandedItems = [];
        
        var oldSelectedTreeItems = SelectedTreeItems.Select(i => i.AbsolutePath).ToList();
        List<TreeItemViewModel> newSelectedTreeItems = [];
        
        var root = new TreeItemViewModel(this)
        {
            Name = WorkspaceRoot.GetFileName(),
            AbsolutePath = WorkspaceRoot,
            IsExpanded = oldExpandedItems.Contains(WorkspaceRoot),
        };

        if (oldSelectedTreeItems.Contains(WorkspaceRoot))
        {
            newSelectedTreeItems.Add(root);
        }


        
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
                                
                                var isExpanded = oldExpandedItems.Contains(itemPath);

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

                                if (oldSelectedTreeItems.Contains(item.AbsolutePath))
                                {
                                    newSelectedTreeItems.Add(item);
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
        else
        {
            SelectedTreeItems = new ObservableCollection<TreeItemViewModel>(newSelectedTreeItems);
        }

    }


    [RelayCommand]
    private async Task Patch()
    {
        if (!IsPatchButtonEnable)
        {
            return;
        }


        var file = await Manager.Default.Send(new OnFilePickerOpen()
        {
            Options = new FilePickerOpenOptions()
            {
                AllowMultiple = false,
                Title = "Select a patch file"
            }
        }, Manager.MainWindowToken);

        if (file.Count == 0)
        {
            return;
        }

        var patchDialogModel = new PatchDialogModel(this)
        {
            Target = SelectedTreeItems.First().AbsolutePath,
            Path = file[0].Path.AbsolutePath
        };

        var hostId = SendMessage(new OnGetDialogHostId());
        
        var dialogOptions = new OverlayDialogOptions
        {
            FullScreen = true,
            Title = "Patch",
            IsCloseButtonVisible = false,
            StyleClass = "Fixed"
        };
        
        await OverlayDialog.ShowModal<PatchDialog, PatchDialogModel>(patchDialogModel, hostId, dialogOptions);
    }

    [RelayCommand]
    private async Task Mkdir()
    {
        if (!IsMkdirButtonEnable)
        {
            return;
        }

        try
        {
            var mkdirDialogModel = new MkdirDialogModel()
            {
                ParentDirectory = SelectedTreeItems.First().AbsolutePath,
            };

            var dialogOptions = new OverlayDialogOptions()
            {
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None,
                CanDragMove = true,
                Mode = DialogMode.Question,
                Title = "Create a versioned directory",
            };
        
            await OverlayDialog.ShowModal<MkdirDialog, MkdirDialogModel>(mkdirDialogModel, SendMessage(new OnGetDialogHostId()), dialogOptions);
        
            var path = mkdirDialogModel.ParentDirectory + "/" + mkdirDialogModel.Name;
        
            var options = new MkdirOptions([path], true, null, string.Empty);
            await _context.Value.Mkdir(options);
            await RefreshTreeItems();
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to create directory:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            });
        }
    }

    [RelayCommand]
    private async Task Refresh()
    {
        if (!IsRefreshButtonEnable)
        {
            return;
        }

        Manager.Default.Send(new OnShowToast()
        {
            Content = $"Refreshing: {SelectedTreeItems.First().AbsolutePath.TrimStartString(WorkspaceRoot?.GetDirectoryName() ?? string.Empty).TrimStartPathSeparatorChar()}",
            Type = NotificationType.Information
        }, Manager.MainWindowToken);
        await RefreshTreeItems();
    }


    [RelayCommand]
    private async Task Revert()
    {
        try
        {
            if (!IsRevertButtonEnable)
            {
                return;
            }

            var revertOptions = new RevertOptions(
                [SelectedTreeItems.First().AbsolutePath],
                Depth.Infinity,
                [],
                false,
                false,
                true
            );
        
        
            await _context.Value.Revert(revertOptions);

            await RefreshTreeItems();
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to revert {SelectedTreeItems.First().AbsolutePath}:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }

    [RelayCommand]
    private async Task Delete()
    {
        if (!IsDeleteButtonEnable)
        {
            return;
        }

        var path = SelectedTreeItems.First().AbsolutePath;

        var deleteOptions = new DeleteOptions([path], false, true, null);

        await _context.Value.Delete(deleteOptions);
        
        await RefreshTreeItems();
    }

    [RelayCommand]
    private async Task Add()
    {
        if (!IsAddButtonEnable)
        {
            return;
        }

        var addOptions = new AddOptions(SelectedTreeItems.First().AbsolutePath, Depth.Infinity, false, false, false, false);

        await _context.Value.Add(addOptions);
        await RefreshTreeItems();
    }

    [RelayCommand]
    private async Task Commit()
    {
        if (!IsCommitButtonEnable)
        {
            return;
        }

        var commitDialogModel = new CommitDialogModel(this)
        {
            Targets = SelectedTreeItems.Where(i => i.StatusEntry is not null).Select(i => i.StatusEntry!).ToArray()
        };
        var hostId = SendMessage(new OnGetDialogHostId());

        var dialogOptions = new OverlayDialogOptions()
        {
            StyleClass = "Fixed",
            IsCloseButtonVisible = false,
            Buttons = DialogButton.None,
            CanDragMove = true,
        };
        
        await OverlayDialog.ShowModal<CommitDialog, CommitDialogModel>(commitDialogModel, hostId, dialogOptions);
    }


    private async Task Prepare()
    {
        try
        {
            var root = await _context.Value.GetWcRoot(WorkspacePath);

            var found = SendMessage(new OnGetTabByWorkspaceRoot()
            {
                Root = root
            }).Response;
            if (found is not null)
            {
                var hostId = SendMessage(new OnGetDialogHostId());
                var result = await MessageBox.ShowOverlayAsync($"The root of {WorkspacePath} is already opened. Redirect to the specified page?", title: "Info", hostId: hostId, icon: MessageBoxIcon.Question, MessageBoxButton.YesNo);
                if (result == MessageBoxResult.Yes)
                {
                    if (!SendMessage(new OnSwitchTab()
                        {
                            Tab = found,
                        }))
                    {
                        Manager.Default.Send(new OnShowToast()
                        {
                            Content = "Failed to redirect to the specified page,\nmaybe the page is closed",
                            Type = NotificationType.Error
                        }, Manager.MainWindowToken);
                    }
                }
                SendMessage(new OnRemoveTabModel());
            }

            WorkspaceRoot = root;
        }
        catch (System.Exception e)
        {
            await e.HandleAsync(svnExceptionHandler: async error =>
            {
                var constant = new SvnErrnoConstants();
                if (constant.IsWcNotWorkingCopy(error.Code))
                {
                    var hostId = SendMessage(new OnGetDialogHostId());

                    var result = await MessageBox.ShowOverlayAsync($"{WorkspacePath} is not working copy,\nWhether to initialize now?", title: "Error", hostId: hostId, icon: MessageBoxIcon.Error, MessageBoxButton.YesNo);
                    if (result == MessageBoxResult.Yes)
                    {
                        
                    }
                    else
                    {
                        SendMessage(new OnRemoveTabModel());
                    }

                    return true;
                }
                return false;
            });
        }
        finally
        {
            
        }
    }
    
    [RelayCommand]
    private async Task OnLoaded()
    {
        var queue = SendMessage(new OnGetSingleTaskQueue()).Response;
        await queue.Run(async token =>
        {
            await Prepare();
            await RefreshTreeItems(true);
        });
    }

    [RelayCommand]
    private async Task Difference()
    {
        var differenceDialogModel = new DifferenceDialogModel(this)
        {
            Path = SelectedTreeItems.First().AbsolutePath,
        };

        var dialogOptions = new OverlayDialogOptions
        {
            FullScreen = true,
            Title = "Difference",
            IsCloseButtonVisible = false,
            StyleClass = "Fixed"
        };

        await OverlayDialog.ShowModal<DifferenceDialog, DifferenceDialogModel>(differenceDialogModel, SendMessage(new OnGetDialogHostId()), dialogOptions);
    }

    [RelayCommand]
    private async Task Update()
    {
        try
        {
            if (SelectedTreeItem is null)
            {
                Manager.Default.Send(new OnShowToast
                {
                    Type = NotificationType.Warning,
                    Content = "No selected item to update"
                }, Manager.MainWindowToken);
                return;
            }

            var updateOptions = new UpdateOptions(
                [SelectedTreeItem.AbsolutePath],
                new Revision.Head(),
                Depth.Infinity,
                false,
                false,
                true,
                true,
                true
            );
        
            var revision = (await _context.Value.Update(updateOptions)).FirstOrDefault();

            Manager.Default.Send(new OnShowToast()
            {
                Type =  NotificationType.Success,
                Content = $"Updated Successfully to r{revision}"
            }, Manager.MainWindowToken);

        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast
            {
                Content = "Failed to update the workspace.",
                Type =  NotificationType.Error,
            }, Manager.MainWindowToken);
        }
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

    public void Receive(OnGetContext message)
    {
        message.Reply(_context.Value);
    }

    public void Receive(OnGetWorkingCopyRoot message)
    {
        message.Reply(WorkspaceRoot ?? string.Empty);
    }
}