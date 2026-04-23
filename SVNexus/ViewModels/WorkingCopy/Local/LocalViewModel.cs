using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalViewModel : ViewModelBase, IRecipient<LocalViewModel.OnCreatedItem>
{


    public class OnCreatedItem(TreeItemViewModel value) : ValueChangedMessage<TreeItemViewModel>(value);
    
    
    public partial class TreeItemViewModel(ViewModelBase? parent): ViewModelBase(parent)//, IRecipient<OnSetChecked>, IRecipient<OnSetExpanded>
    {
        
        [ObservableProperty]
        public partial bool HasLoaded { get; set; }
        
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }

        [ObservableProperty]
        public partial ObservableCollection<TreeItemViewModel> Children { get; set; } = [];
        
        public ObservableCollection<MenuItemViewModel> MenuItems { get; set; } = [];

        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(HasChild))]
        [NotifyPropertyChangedFor(nameof(IsDelete))]
        [NotifyPropertyChangedFor(nameof(NodeKindIcon))]
        [NotifyPropertyChangedFor(nameof(Name))]
        [NotifyPropertyChangedFor(nameof(StatusToolTip))]
        [NotifyPropertyChangedFor(nameof(StatusIcon))]
        [NotifyPropertyChangedFor(nameof(IsLocked))]
        public required partial StatusEntry StatusEntry { get; set; }
        
        public bool HasChild => StatusEntry.NodeKind == NodeKind.Directory;
        
        // public bool IsReal => StatusEntry.NodeStatus is not (WorkingCopyStatus.None or WorkingCopyStatus.Normal);
        
        public bool IsLocked =>  StatusEntry.Lock is not null;
        
        public bool IsDelete => StatusEntry.NodeStatus == WorkingCopyStatus.Deleted;

        [ObservableProperty] public partial bool IsLoading { get; set; }

        public string NodeKindIcon => StatusEntry.NodeKind.Icon();


        public string Name => StatusEntry.Path.GetFileName();


        public string StatusToolTip => StatusEntry.NodeStatus.ToString();


        public string StatusIcon => StatusEntry.NodeStatus.Icon();
        

        partial void OnIsExpandedChanged(bool value)
        {
            if (value && !HasLoaded)
            {
                _ = LoadChildren();
            }
        }

        public async Task RefreshChildren(bool includeSelf = false)
        {
            if (IsLoading || (!HasLoaded && !includeSelf))
            {
                return;
            }
            IsLoading = true;
            Logger.Info($"{StatusEntry.Path} Loading children...");
            try
            {

                // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
                var hostId = SendMessage(new OnGetDialogHostId());

                var context = EngineBackend.Instance.SimpleContext(hostId);

                var statusOptions = new StatusOptions(
                    StatusEntry.Path, 
                    new Revision.Working(), 
                    HasLoaded ? Depth.Immediates : Depth.Empty, 
                    true,
                    false, 
                    false, 
                    false, 
                    false, 
                    false, 
                    null);

                var children = new List<TreeItemViewModel>();
                var receiver = new StatusReceiverDelegate()
                {
                    OnStatusEntryAction = entry =>
                    {
                        try
                        {
                            Dispatcher.UIThread.Invoke(() =>
                            {
                                if (entry.Path == StatusEntry.Path)
                                {
                                    if (includeSelf)
                                    {
                                        StatusEntry = entry;
                                    }
                                    return;
                                }

                                var index = Children.FindIndex(i => i.StatusEntry.Path == entry.Path);
                                if (index >= 0)
                                {
                                    Children[index].StatusEntry = entry;
                                    Children[index].Parent = Parent;
                                }
                                else
                                {
                                    var item = new TreeItemViewModel(Parent)
                                    {
                                        StatusEntry = entry
                                    };
                                    children.Add(item);
                                    
                                    SendMessage(new OnCreatedItem(item));
                                }

                            });
                        }
                        catch (System.Exception e)
                        {
                            Console.WriteLine(e);
                        }
                    }
                };
                
                foreach (var child in Children)
                {
                    child.Parent = null;
                }

                await context.StatusNext(statusOptions, receiver);

                for (var i = Children.Count - 1; i >= 0; i--)
                {
                    if (Children[i].Parent is null)
                    {
                        Children.RemoveAt(i);
                    }
                }

                Children.AddRange(children);
                
                Logger.Info($"Loaded children: {Children.Count}");
                
                // HasLoaded = true;
            }
            catch (System.Exception e)
            {
                Console.WriteLine(e);
            }
            finally
            {
                IsLoading = false;
            }
            
        }


        public async Task LoadChildren()
        {
            if (HasLoaded || IsLoading || !HasChild)
            {
                return;
            }
            IsLoading = true;
            // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
            
            var hostId = SendMessage(new OnGetDialogHostId());
            
            var context = EngineBackend.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(StatusEntry.Path, new Revision.Working(), Depth.Immediates, true, false, false, false, false, false, null);

            var receiver = new StatusReceiverDelegate()
            {
                OnStatusEntryAction = entry =>
                {
                    try
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            if (entry.Path == StatusEntry.Path)
                            {
                                return;
                            }


                            var item = new TreeItemViewModel(Parent)
                            {
                                StatusEntry = entry,
                            };
                            Children.Add(item);
                            SendMessage(new OnCreatedItem(item));
                        });
                    }
                    catch (System.Exception e)
                    {
                        Console.WriteLine(e);
                    }
                }
            };
            
            await context.StatusNext(statusOptions, receiver);

            HasLoaded = true;
            IsLoading = false;
        }
    }

    private readonly LimitedDictionary<string, DifferenceViewModel> _differences = new()
    {
        Limit = 20
    };
    
    private readonly List<WeakReference<TreeItemViewModel>> _weakTreeItems = [];

    [ObservableProperty]
    public partial DifferenceViewModel DifferenceViewModel { get; set; }

    public ObservableCollection<TreeItemViewModel> TreeItems
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
    public partial TreeItemViewModel? SelectedTreeItem { get; set; }
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(TreeItems))]
    public partial bool ShowRoot { get; set; }
    
    [ObservableProperty]
    public partial bool IsLoading { get; set; }
    
    
    
    [ObservableProperty]
    public partial ObservableCollection<TreeItemViewModel> SelectedTreeItems { get; set; } = [];
    
    
    public bool IsAddButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry.NodeStatus == WorkingCopyStatus.Unversioned);

    public bool IsUpdateButtonEnable => SelectedTreeItems.Count > 0;

    public bool IsRefreshButtonEnable => true;

    public bool IsRevertButtonEnable => SelectedTreeItems.Count > 0;
    
    public bool IsDiffButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry.NodeStatus != WorkingCopyStatus.Unversioned;
    
    public bool IsPatchButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry.NodeKind == NodeKind.Directory && SelectedTreeItems.First().StatusEntry.NodeStatus != WorkingCopyStatus.Unversioned;

    public bool IsDeleteButtonEnable => SelectedTreeItems.Count > 0;
    
    public bool IsMkdirButtonEnable => SelectedTreeItems.Count == 1 && SelectedTreeItems.First().StatusEntry.NodeKind == NodeKind.Directory && SelectedTreeItems.First().StatusEntry.NodeStatus != WorkingCopyStatus.Unversioned;
    
    public bool IsLockButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry is { NodeKind: NodeKind.File, Lock: null });
    
    public bool IsUnlockButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry.NodeKind != NodeKind.Directory && i.StatusEntry.Lock is not null);
    
    public bool IsCommitButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.Any(i => i.StatusEntry?.NodeStatus != WorkingCopyStatus.Unversioned);


    // private Dictionary<string, StatusEntry> _checkedItems = [];
    // private HashSet<string> _expandedItems = [];

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(TreeItems))]
    public partial TreeItemViewModel? Root { get; set; }

    private readonly SingleTaskQueue _singleTaskQueue = new();

    /// <inheritdoc/>
    public LocalViewModel(ViewModelBase parent) : base(parent)
    {
        SelectedTreeItems?.CollectionChanged += (sender, args) =>
        {
            NotifyProperties();
        };

        DifferenceViewModel = new DifferenceViewModel(this);
    }

    partial void OnSelectedTreeItemChanged(TreeItemViewModel? value)
    {
        if (value is null)
        {
            DifferenceViewModel = new DifferenceViewModel(this);
        }
        else
        {
            if (_differences.TryGetValue(value.StatusEntry.Path, out var model))
            {
                DifferenceViewModel = model;
            }
            else
            {
                DifferenceViewModel = new DifferenceViewModel(this);
                Dispatcher.UIThread.InvokeAsync(async () =>
                {
                    await DifferenceViewModel.CompareWorkingCopyEntry(value.StatusEntry);
                });
                
                _differences.Add(value.StatusEntry.Path, DifferenceViewModel);
            }
        }
    }


    private void NotifyProperties()
    {
        OnPropertyChanged(nameof(IsAddButtonEnable));
        OnPropertyChanged(nameof(IsUpdateButtonEnable));
        OnPropertyChanged(nameof(IsRevertButtonEnable));
        OnPropertyChanged(nameof(IsRevertButtonEnable));
        OnPropertyChanged(nameof(IsDiffButtonEnable));
        OnPropertyChanged(nameof(IsPatchButtonEnable));
        OnPropertyChanged(nameof(IsDeleteButtonEnable));
        OnPropertyChanged(nameof(IsMkdirButtonEnable));
        OnPropertyChanged(nameof(IsLockButtonEnable));
        OnPropertyChanged(nameof(IsUnlockButtonEnable));
        OnPropertyChanged(nameof(IsCommitButtonEnable));
    }

    // public async Task ExecuteRefreshCommand()
    // {
    //     if (RefreshCommand.CanExecute(null))
    //     {
    //         await RefreshCommand.ExecuteAsync(null);
    //     }
    // }

    [RelayCommand]
    private async Task Commit()
    {
        if (!IsCommitButtonEnable)
        {
            return;
        }

        var commitDialogModel = new CommitDialogModel(this)
        {
            Targets = SelectedTreeItems.Select(i => i.StatusEntry).ToArray(),
            RelateTo = SendMessage(new OnGetWorkingCopyPath())
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

    [RelayCommand]
    private async Task Mkdir()
    {
        if (!IsMkdirButtonEnable)
        {
            return;
        }

        var mkdirDialogModel = new MkdirDialogModel(this)
        {
            ParentDirectory = SelectedTreeItems.First().StatusEntry.Path,
        };
            
        await OverlayDialog.ShowModal<MkdirDialog, MkdirDialogModel>(mkdirDialogModel, SendMessage(new OnGetDialogHostId()), mkdirDialogModel.OverlayDialogOptions);
        
        if (mkdirDialogModel.Accept)
        {
            await RefreshCommand.ExecuteOrNothingAsync(null);
        }

    }

    [RelayCommand]
    private async Task Delete()
    {
        if (!IsDeleteButtonEnable)
        {
            return;
        }
        
        var deleteOptions = new DeleteOptions(SelectedTreeItems.Select(i => i.StatusEntry.Path).ToArray(), false,  false,  null);

        var context = SendMessage(new OnGetContext()).Response;
        
        await context.Delete(deleteOptions);

        await RefreshCommand.ExecuteOrNothingAsync(null);
    }

    [RelayCommand]
    private async Task Unlock()
    {
        if (!IsUnlockButtonEnable)
        {
            return;
        }

        // foreach (var item in SelectedTreeItems)
        // {
        //     Logger.Info($"{item.StatusEntry.Path} is {item.StatusEntry.Lock is null}");
        // }
        //
        var unlockDialogModel = new UnlockDialogModel(this)
        {
            RelateTo = SendMessage(new OnGetWorkingCopyPath()),
            TargetEntries = SelectedTreeItems.Select(i => i.StatusEntry).ToList()
        };

        var dialogOptions = new OverlayDialogOptions()
        {
            IsCloseButtonVisible = false,
            Buttons = DialogButton.None,
            CanDragMove = true,
            StyleClass = "Fixed",
            Title = "Unlock"
        };
        
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<UnlockDialog, UnlockDialogModel>(unlockDialogModel, hostId, dialogOptions);
        
        Logger.Info($"Unlock complete: {unlockDialogModel.Accept}");

        if (unlockDialogModel.Accept)
        {
            await RefreshCommand.ExecuteOrNothingAsync(null);
            // await ExecuteRefreshCommand();
            // NotifyProperties();
        }

    }

    [RelayCommand]
    private async Task Lock()
    {
        if (!IsLockButtonEnable)
        {
            return;
        }
        var lockDialogModel = new LockDialogModel(this)
        {
            RelateTo = SendMessage(new OnGetWorkingCopyPath()),
            TargetEntries = SelectedTreeItems.Select(i => i.StatusEntry).ToList()
        };

        var dialogOptions = new OverlayDialogOptions()
        {
            IsCloseButtonVisible = false,
            Buttons = DialogButton.None,
            CanDragMove = true,
            StyleClass = "Fixed",
            Title = "Lock"
        };
        
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<LockDialog, LockDialogModel>(lockDialogModel, hostId, dialogOptions);
        if (lockDialogModel.Accept)
        {
            await Refresh();
            // NotifyProperties();
        }
        // var lockOptions = new LockOptions();
    }


    [RelayCommand]
    private async Task Difference()
    {
        if (!IsDiffButtonEnable)
        {
            return;
        }
        
        var differenceDialogModel = new DifferenceDialogModel(this)
        {
            Path = SelectedTreeItems.First().StatusEntry.Path,
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
    private async Task Revert()
    {
        try
        {
            if (!IsRevertButtonEnable)
            {
                return;
            }

            var hostId = SendMessage(new OnGetDialogHostId());
            var context = SendMessage(new OnGetContext()).Response;


            var result = await MessageBox.ShowOverlayAsync("This operation cannot be undone. Do you want to continue?", "Warning", hostId, MessageBoxIcon.Warning, MessageBoxButton.YesNo);
            if (result != MessageBoxResult.Yes)
            {
                return;
            }

            var revertOptions = new RevertOptions(
                SelectedTreeItems.Select(i => i.StatusEntry.Path).ToArray(),
                Depth.Infinity,
                null,
                false,
                false,
                true
            );
        
        
            await context.Revert(revertOptions);

            await Refresh();
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to revert:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }

    [RelayCommand]
    private async Task Update()
    {
        if (!IsUpdateButtonEnable)
        {
            return;
        }
        
        var updateOptions = new UpdateOptions(
            SelectedTreeItems.Select(i => i.StatusEntry.Path).ToArray(),
            new Revision.Head(),
            Depth.Infinity,
            false,
            false,
            true,
            true,
            true
        );


        var context = SendMessage(new OnGetContext()).Response;

        await context.Update(updateOptions);
        await Refresh();
    }

    [RelayCommand]
    private async Task Add()
    {
        if (!IsAddButtonEnable)
        {
            return;
        }
        var context = SendMessage(new OnGetContext()).Response;

        foreach (var item in SelectedTreeItems)
        {
            var addOptions = new AddOptions(item.StatusEntry.Path, Depth.Infinity, false, false, false, true);

            await context.Add(addOptions);
        }
        await Refresh();
    }

    private void CleanWeakTreeItems()
    {
        ForEachWeakTreeItem(_ => {});
    }

    private void ForEachWeakTreeItem(Action<TreeItemViewModel> action)
    {
        for (var i = _weakTreeItems.Count - 1; i >= 0; i--)
        {
            if (_weakTreeItems[i].TryGetTarget(out var item))
            {
                action(item);
            }
            else
            {
                _weakTreeItems.RemoveAt(i);
            }
        }
    }

    [RelayCommand]
    private void CollapseAll()
    {
        // Manager.Default.Send(new OnSetExpanded(false), _typeService.Get<TreeItemViewModel>());
        ForEachWeakTreeItem(item => item.IsExpanded = false);
    }

    [RelayCommand]
    private void ExpandAll()
    {
        // Manager.Default.Send(new OnSetExpanded(true), _typeService.Get<TreeItemViewModel>());
        ForEachWeakTreeItem(item => item.IsExpanded = true);
    }

    // partial void OnShowRootChanged(bool value)
    // {
    //     if (Root is null)
    //     {
    //         return;
    //     }
    //     
    //     TreeItems.Clear();
    //     if (value)
    //     {
    //         TreeItems.Add(Root);
    //     }
    //     else
    //     {
    //         TreeItems.AddRange(Root.Children);
    //     }
    // }

    private async Task RefreshItem(TreeItemViewModel item)
    {
        await item.RefreshChildren(true);
        foreach (var child in item.Children)
        {
            _ = Dispatcher.UIThread.InvokeAsync(async () =>
            {
                await RefreshItem(child);
            });
        }
    }

    
    [RelayCommand]
    private async Task RefreshSelectedItem()
    {
        if (SelectedTreeItems.Count == 0)
        {
            await Refresh();
        }
        else
        {
            foreach (var item in SelectedTreeItems)
            {
                if (item.Parent == this)
                {
                    await item.RefreshChildren(true);
                }
            }
            NotifyProperties();
        }
        // OnShowRootChanged(ShowRoot);
    }
    
    [RelayCommand]
    private async Task Refresh()
    {
        await _singleTaskQueue.RunAndWait(async token =>
        {
            if (Root is null)
            {
                if (!IsLoading)
                {
                    token.ThrowIfCancellationRequested();
                    await LoadRoot();
                    NotifyProperties();
                }
            }
            token.ThrowIfCancellationRequested();

            if (Root is not null)
            {
                await RefreshItem(Root);
                NotifyProperties();
            }
            CleanWeakTreeItems();
            // OnShowRootChanged(ShowRoot);
        });
        
    }

    private async Task LoadAllChildren(TreeItemViewModel item)
    {
        if (item is { HasLoaded: false, HasChild: true })
        {
            await item.LoadChildren();
        }

        foreach (var child in item.Children)
        {
            _ = Dispatcher.UIThread.InvokeAsync(async () =>
            {
                await LoadAllChildren(child);
            });
        }
    }

    [RelayCommand]
    private async Task LoadAll()
    {
        await _singleTaskQueue.Run(async token =>
        {
            if (Root is null && !IsLoading)
            { 
                token.ThrowIfCancellationRequested();
                await LoadRoot();
            }

            if (Root is null)
            {
                return;
            }
            token.ThrowIfCancellationRequested();
            await LoadAllChildren(Root);
            CleanWeakTreeItems();
        }, false);
    }


    private async Task LoadRoot()
    {
        IsLoading = true;
        try
        {
            // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
            var hostId = SendMessage(new OnGetDialogHostId());
            var path = SendMessage(new OnGetWorkingCopyPath());

            var context = EngineBackend.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(
                path,
                new Revision.Working(),
                Depth.Immediates,
                true,
                false,
                false,
                false,
                true,
                false,
                null);

            var result = await context.Status(statusOptions);

            var children = new List<TreeItemViewModel>();

            foreach (var entry in result.Entries)
            {
                if (entry.Path == path)
                {
                    Root = new TreeItemViewModel(this)
                    {
                        StatusEntry = entry,
                    };
                    
                    Receive(new OnCreatedItem(Root));
                }
                else
                {
                    children.Add(new TreeItemViewModel(this)
                    {
                        StatusEntry = entry,
                    });
                    
                    Receive(new OnCreatedItem(children.Last()));
                }
            }

            Root?.Children.AddRange(children);
            Root?.HasLoaded = true;
            Root?.IsExpanded = true;
            // OnShowRootChanged(ShowRoot);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to load working copy: {e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
        finally
        {
            IsLoading = false;
        }
    }

    // [RelayCommand]
    // private async Task OnLoaded()
    // {
    //     await RefreshCommand.ExecuteOrNothingAsync(null);
    // }

    public void Receive(OnCreatedItem message)
    {
        _weakTreeItems.Add(new WeakReference<TreeItemViewModel>(message.Value));
    }

    [RelayCommand]
    private async Task OnShow()
    {
        await RefreshCommand.ExecuteOrNothingAsync(null);
    }
}