using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Collections.Specialized;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Controls.Primitives;
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
using Exception = System.Exception;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkspaceViewModel : ViewModelBase, 
    IRecipient<WorkspaceViewModel.OnItemIsExpandedChanged>,
    IRecipient<OnGetContext>,
    IRecipient<OnGetWorkingCopyRoot>,
    IRecipient<OnGetWorkspaceHistory>,
    IRecipient<OnSetWorkspaceHistory>,
    IRecipient<OnNotWorkingCopy>
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


        public string NodeKindIcon => StatusEntry?.NodeKind.Icon() ?? string.Empty;
        
        public string NodeStatusIcon => StatusEntry?.NodeStatus.Icon() ?? string.Empty;

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


    public WorkspaceHistory.WorkingCopy? History { get; set; }
    
    
    [ObservableProperty]
    public partial TreeItemViewModel? SelectedTreeItem { get; set; }

    [ObservableProperty]
    public partial ObservableCollection<TreeItemViewModel> SelectedTreeItems { get; set; } = [];
    
    

    [ObservableProperty]
    public partial WorkingCopyViewModel? WorkingCopyViewModel { get; set; }
    
    public ObservableCollection<TreeItemViewModel> TreeItems { get; set; } = [];
    
    private List<string> _expandedItems = [];

    public bool IsAddButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry?.NodeStatus == WorkingCopyStatus.Unversioned);

    public bool IsUpdateButtonEnable => SelectedTreeItems.Count > 0;

    // public bool IsRefreshButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    public bool IsRefreshButtonEnable => true;
    
    // public bool IsRevertButtonEnable =>  SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != NodeStatus.Unversioned;
    public bool IsRevertButtonEnable => SelectedTreeItems.Count > 0;
    
    public bool IsDiffButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeKind == NodeKind.Directory && SelectedTreeItems.First().StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned;
    
    public bool IsPatchButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeKind == NodeKind.Directory && SelectedTreeItems.First().StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned;
    
    // public bool IsDeleteButtonEnable => SelectedTreeItems.Count == 1 && 
    //                                     SelectedTreeItems.First().StatusEntry?.NodeStatus is not (NodeStatus.Unversioned or NodeStatus.Added) &&
    //                                     SelectedTreeItems.First().AbsolutePath != WorkspaceRoot;
    
    public bool IsDeleteButtonEnable => SelectedTreeItems.Count > 0; // && SelectedTreeItems.All(i => i.StatusEntry is not null && i.StatusEntry.NodeStatus != NodeStatus.Unversioned);
    
    public bool IsMkdirButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned;
    
    public bool IsCommitButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.Any(i => i.StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned);
    
    public bool IsSwitchButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned;
    
    public bool IsMergeButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned;
    
    public bool IsRelocateButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().AbsolutePath == WorkspaceRoot;
    
    public bool IsOpenTerminalButtonEnable => SelectedTreeItems.Count == 1;

    public bool IsInfoButtonEnable => SelectedTreeItems.Count == 1;
    

    /// <inheritdoc/>
    public WorkspaceViewModel(string path, ViewModelBase parent) : base(parent)
    {
        WorkspacePath = path;

        _context = new Lazy<AsyncContext>(() =>
        {
            var hostId = SendMessage(new OnGetDialogHostId()); 
            var context = EngineBackend.Instance.SimpleContext(hostId);
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
        OnPropertyChanged(nameof(IsOpenTerminalButtonEnable));
        OnPropertyChanged(nameof(IsInfoButtonEnable));
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
        if (value is null || value.StatusEntry?.NodeStatus == WorkingCopyStatus.Unversioned)
        {
            WorkingCopyViewModel = null;
            return;
        }


        if (History is not null)
        {
            
            EngineBackend.Instance.DatabaseQueue.Run(async _ =>
            {
                await SeaDatabaseConnection.Default.UpdateWorkspaceHistory(History.Uuid, new UpdateOperationDelegate()
                {
                    UpdateFunc = w =>
                    {
                        var v = w.ToWorkspaceHistory();
                        if (v is not WorkspaceHistory.WorkingCopy wc) return w;
                        v = wc with { WorkingCopyPath = value.AbsolutePath };
                        return AnyValue.FromWorkspaceHistory(v);
                    }
                });
            });
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


        var lockedEntries = new List<StatusEntry>();
        var incompleteEntries = new List<StatusEntry>();
        
        TreeItemViewModel? selectedTreeItem = null;

        var receiver = new StatusReceiverDelegate()
        {
            OnStatusEntryAction = entry =>
            {
                try
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        if (entry.WcIsLocked)
                        {
                            lockedEntries.Add(entry);
                        }

                        if (entry.NodeStatus == WorkingCopyStatus.Incomplete)
                        {
                            incompleteEntries.Add(entry);
                        }
                        if (entry.NodeKind != NodeKind.Directory)
                        {
                            return;
                        }

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
                catch (Exception e)
                {
                    Console.WriteLine(e);
                }
            }
        };

        await _context.Value.StatusNext(statusOptions, receiver);

        if (lockedEntries.Count > 0)
        {
            var hostId = SendMessage(new OnGetDialogHostId());

            var result = await OverlayMessageBox.ShowAsync(
                "Working copy is locked, Whether to cleanup", 
                "Error", 
                hostId,  
                MessageBoxIcon.Error, 
                MessageBoxButton.YesNo);
            if (result == MessageBoxResult.Yes)
            {
                var options = new CleanupOptions(WorkspaceRoot, true, true, true, true, true);
                
                await _context.Value.Cleanup(options);
                
                await RefreshTreeItems(initialize);
            }
            else
            {
                SendMessage(new OnRemoveTabModel());
            }
            return;
        }

        if (incompleteEntries.Count > 0)
        {
            var hostId = SendMessage(new OnGetDialogHostId());
            var result = await OverlayMessageBox.ShowAsync("Working copy is locked, Whether to update", "Error", hostId,  MessageBoxIcon.Error, MessageBoxButton.YesNo);
            if (result == MessageBoxResult.Yes)
            {

                var options = new UpdateOptions([WorkspaceRoot], new Revision.Base(), Depth.Infinity, false, false, true, true, true);
                
                await _context.Value.Update(options);
                
                await RefreshTreeItems(initialize);
            }
            else
            {
                SendMessage(new OnRemoveTabModel());
            }

            return;
        }
        
        
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
    private async Task Info()
    {
        if (!IsInfoButtonEnable)
        {
            return;
        }

        var model = new InfoDialogModel(this)
        {
            Path = SelectedTreeItems.First().AbsolutePath,
            PegRevision = new Revision.Unspecified(),
            Revision =  new Revision.Unspecified(),
        };

        await model.LoadedCommand.ExecuteOrNothingAsync(null);

        if (model.Entry is null)
        {
            return;
        }

        var hostId = SendMessage(new OnGetDialogHostId());

        _ = Dispatcher.UIThread.InvokeAsync(async () =>
        {
            await OverlayDialog.ShowStandardAsync<InfoDialog, InfoDialogModel>(model, hostId, model.OverlayDialogOptions);
        });
    }

    [RelayCommand]
    private async Task OpenTerminal()
    {
        if (!IsOpenTerminalButtonEnable)
        {
            return;
        }
        
        await IPlatform.Create().OpenTerminal(SelectedTreeItems.First().AbsolutePath);
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
        
        await OverlayDialog.ShowStandardAsync<PatchDialog, PatchDialogModel>(patchDialogModel, hostId, patchDialogModel.OverlayDialogOptions);
    }

    [RelayCommand]
    private async Task Mkdir()
    {
        if (!IsMkdirButtonEnable)
        {
            return;
        }

        var mkdirDialogModel = new MkdirDialogModel(this)
        {
            ParentDirectory = SelectedTreeItems.First().AbsolutePath,
        };

        await OverlayDialog.ShowStandardAsync<MkdirDialog, MkdirDialogModel>(mkdirDialogModel, SendMessage(new OnGetDialogHostId()), mkdirDialogModel.OverlayDialogOptions);
    
        if (mkdirDialogModel.Accept)
        {
            await RefreshTreeItems();
        }
    }

    [RelayCommand]
    private async Task Refresh()
    {
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

            var hostId = SendMessage(new OnGetDialogHostId());


            // var result = await MessageBox.ShowOverlayAsync("This operation cannot be undone. Do you want to continue?", "Warning", hostId, MessageBoxIcon.Warning, MessageBoxButton.YesNo);
            // if (result != MessageBoxResult.Yes)
            // {
            //     return;
            // }
            //
            // var revertOptions = new RevertOptions(
            //     SelectedTreeItems.Select(i => i.AbsolutePath).ToArray(),
            //     Depth.Infinity,
            //     false,
            //     false
            // );
            //
            //
            // await _context.Value.Revert(revertOptions);
            //

            var model = new RevertDialogModel(this)
            {
                Targets = SelectedTreeItems.Select(i => TargetItemViewModel.From(i.StatusEntry!, false, WorkspaceRoot)).ToList()
            };

            await OverlayDialog.ShowStandardAsync<RevertDialog, RevertDialogModel>(model, hostId, model.OverlayDialogOptions);
            if (model.Accept)
            {
                await RefreshTreeItems();
            }
        }
        catch (Exception e)
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

        try
        {

            var model = new DeleteDialogModel(this)
            {
                Targets = SelectedTreeItems.Select(i => TargetItemViewModel.From(i.StatusEntry!, false, WorkspaceRoot)).ToList()
            };
            
            await OverlayDialog.ShowStandardAsync<DeleteDialog, DeleteDialogModel>(model, SendMessage(new OnGetDialogHostId()), model.OverlayDialogOptions);

            // var deleteOptions = new DeleteOptions(SelectedTreeItems.Select(i => i.AbsolutePath).ToArray(), false, false, null);
            //
            // await _context.Value.Delete(deleteOptions);
        }
        catch (Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to delete: {e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }

        
        await RefreshTreeItems();
    }

    [RelayCommand]
    private async Task Add()
    {
        if (!IsAddButtonEnable)
        {
            return;
        }

        foreach (var treeItem in SelectedTreeItems)
        {
            var addOptions = new AddOptions(treeItem.AbsolutePath, Depth.Infinity, false, false, false, true);
            await _context.Value.Add(addOptions);
        }

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
            Targets = SelectedTreeItems.Where(i => i.StatusEntry is not null).Select(i => i.StatusEntry!).ToArray(),
            RelateTo = WorkspaceRoot ?? string.Empty
        };
        var hostId = SendMessage(new OnGetDialogHostId());

        // var dialogOptions = new OverlayDialogOptions()
        // {
        //     StyleClass = "Fixed",
        //     IsCloseButtonVisible = false,
        //     Buttons = DialogButton.None,
        //     CanDragMove = true,
        //     HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        //     VerticalScrollBarVisibility = ScrollBarVisibility.Disabled
        // };
        
        await OverlayDialog.ShowStandardAsync<CommitDialog, CommitDialogModel>(commitDialogModel, hostId, commitDialogModel.OverlayDialogOptions);
        if (commitDialogModel.Accept)
        {
            await RefreshTreeItems();
        }
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
                var result = await OverlayMessageBox.ShowAsync($"The root of {WorkspacePath} is already opened. Redirect to the specified page?", title: "Info", hostId: hostId, icon: MessageBoxIcon.Question, MessageBoxButton.YesNo);
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

            var repositoryRoot = await _context.Value.GetRepositoryRoot(WorkspacePath);

            await EngineBackend.Instance.DatabaseQueue.Run(async _ =>
            {
                var historyItems = await SeaDatabaseConnection.Default.WorkspaceHistories();

                var time = DateTimeOffset.UtcNow.ToUnixTimeSeconds();
                if (History is not null)
                {
                    if (historyItems.Any(historyItem => historyItem.Uuid == History.Uuid))
                    {
                        var item = History with { LastUsedTime = time, WorkingCopyPath = WorkspacePath, WorkingCopyRoot = root};
                        // await SeaDatabaseConnection.Default.SetWorkspaceHistory(item);
                        await SeaDatabaseConnection.Default.UpdateWorkspaceHistory(History.Uuid, new UpdateOperationValue(AnyValue.FromWorkspaceHistory(item)));
                        History = item;
                        return;
                    }
                }
                
                foreach (var historyItem in historyItems)
                {
                    if (historyItem is not WorkspaceHistory.WorkingCopy workingCopy ||
                        workingCopy.WorkingCopyRoot != WorkspaceRoot) continue;
                    workingCopy = workingCopy with { LastUsedTime = time };
                    
                    // await SeaDatabaseConnection.Default.SetWorkspaceHistory(workingCopy);
                    
                    await SeaDatabaseConnection.Default.UpdateWorkspaceHistory(workingCopy.Uuid, new UpdateOperationValue(AnyValue.FromWorkspaceHistory(workingCopy)));

                    History = workingCopy;
                    
                    return;
                }


                var history = new WorkspaceHistory.WorkingCopy(WorkspaceRoot, WorkspacePath, repositoryRoot.RootUrl, time, false, 0, false, Guid.NewGuid().ToString(), null);
                await SeaDatabaseConnection.Default.AddWorkspaceHistory(history);
            
                History = history;
            });


        }
        catch (Exception e)
        {
            await e.HandleAsync(svnExceptionHandler: async error =>
            {
                var constant = new SvnErrnoConstants();
                if (!constant.IsWcNotWorkingCopy(error.Code)) return false;
                var hostId = SendMessage(new OnGetDialogHostId());

                var result = await OverlayMessageBox.ShowAsync($"{WorkspacePath} is not working copy,\nWhether to initialize now?", title: "Error", hostId: hostId, icon: MessageBoxIcon.Error, MessageBoxButton.YesNo);
                if (result == MessageBoxResult.Yes)
                {
                    var model = new InitializeRepositoryDialogModel(this)
                    {
                        Local = WorkspacePath,
                    };
                    await model.Show();
                    // var dialogOptions = new OverlayDialogOptions
                    // {
                    //     Title = "Test",
                    //     IsCloseButtonVisible = true,
                    //     Buttons = DialogButton.None
                    // };
                    // var importDialogModel = new ImportDialogModel()
                    // {
                    //     Path = WorkspacePath,
                    // };
                    // await OverlayDialog.ShowStandardAsync<ImportDialog, ImportDialogModel>(importDialogModel, hostId: hostId, options: dialogOptions);
                    // if (importDialogModel.Options is not null)
                    // {
                    //     var importProcessDialogModel = new ImportProcessDialogModel(this)
                    //     {
                    //         Options = importDialogModel.Options,
                    //     };
                    //
                    //     var options = new OverlayDialogOptions
                    //     {
                    //         Title = "Initialize repository",
                    //         IsCloseButtonVisible = false,
                    //         Buttons = DialogButton.None
                    //     };
                    //
                    //     await OverlayDialog.ShowStandardAsync<ImportProcessDialog, ImportProcessDialogModel>(importProcessDialogModel, options: options, hostId: hostId);
                    //     if (importProcessDialogModel.Error is null)
                    //     {
                    //     }
                    // }
                    // else
                    // {
                    //     SendMessage(new OnRemoveTabModel());
                    // }
                }
                else
                {
                    SendMessage(new OnRemoveTabModel());
                }

                return true;
            });
        }
    }
    
    [RelayCommand]
    private async Task OnLoaded()
    {
        var queue = SendMessage(new OnGetSingleTaskQueue()).Response;
        await queue.Run(async _ =>
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

        await OverlayDialog.ShowStandardAsync<DifferenceDialog, DifferenceDialogModel>(differenceDialogModel, SendMessage(new OnGetDialogHostId()), differenceDialogModel.OverlayDialogOptions);
    }

    [RelayCommand]
    private async Task Update()
    {
        try
        {
            if (!IsUpdateButtonEnable)
            {
                return;
            }

            var model = new UpdateDialogModel(this)
            {
                TargetItems = SelectedTreeItems.Where(i => i.StatusEntry is not null).Select(i => TargetItemViewModel.From(i.StatusEntry!)).ToList(),
            };

            var hostId = SendMessage(new OnGetDialogHostId());

            var dialogOptions = new OverlayDialogOptions()
            {
                Title = "Update",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };
            
            
            await OverlayDialog.ShowStandardAsync<UpdateDialog, UpdateDialogModel>(model, hostId, dialogOptions);

            if (model.Accept)
            {
                await RefreshTreeItems();
            }
        }
        catch (Exception e)
        {
            Manager.Default.Send(new OnShowToast
            {
                Content = $"Failed to update the workspace: {e.HumanReadableMessage}",
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

    public void Receive(OnGetWorkspaceHistory message)
    {
        if (History is null)
        {
            return;
        }
        message.Reply(History);
    }

    public void Receive(OnSetWorkspaceHistory message)
    {
        if (History is null)
        {
            return;
        }

        var id = History.Uuid;
        EngineBackend.Instance.DatabaseQueue.Run(async _ =>
        {
            await SeaDatabaseConnection.Default.UpdateWorkspaceHistory(id, new UpdateOperationDelegate()
            {
                UpdateFunc = value =>
                {
                    var v = message.Value.Invoke(value.ToWorkspaceHistory() ?? throw new InvalidOperationException());
                    return AnyValue.FromWorkspaceHistory(v);
                }
            });
        });

    }
    
    
    public void Receive(OnNotWorkingCopy message)
    {
        var hostId = SendMessage(new OnGetDialogHostId());
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            await OverlayMessageBox.ShowAsync(title: "Error", hostId: hostId, message: "Not a working copy", button: MessageBoxButton.OK);
            SendMessage(new OnRemoveTabModel());
        });
    }
}