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

        public required StatusEntry StatusEntry { get; set; }
        
        public bool HasChild => StatusEntry.NodeKind == NodeKind.Directory;
        
        public bool IsReal => StatusEntry.NodeStatus is not (WorkingCopyStatus.None or WorkingCopyStatus.Normal);
        
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

        public async Task RefreshChildren()
        {
            if (IsLoading || !HasLoaded || !HasChild)
            {
                return;
            }
            IsLoading = true;
            try
            {

                // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
                var hostId = SendMessage(new OnGetDialogHostId());

                var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

                var statusOptions = new StatusOptions(StatusEntry.Path, new Revision.Working(), Depth.Immediates, true,
                    true, true, false, false, false, null);

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
                                    return;
                                }

                                var index = Children.FindIndex(i => i.StatusEntry.Path == entry.Path);
                                if (index >= 0)
                                {
                                    Children[index].StatusEntry = entry;
                                    children.Add(Children[index]);
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

                                Children = new ObservableCollection<TreeItemViewModel>(children);
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
            
            var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

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

    // [ObservableProperty]
    // public partial string WorkingCopyPath { get; set; } = string.Empty;

    private readonly List<WeakReference<TreeItemViewModel>> _weakTreeItems = [];
    
    public ObservableCollection<TreeItemViewModel> TreeItems { get; set; } = [];
    
    [ObservableProperty]
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
    
    public bool IsLockButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry.NodeKind == NodeKind.File);
    
    public bool IsUnlockButtonEnable => SelectedTreeItems.Count > 0 && SelectedTreeItems.All(i => i.StatusEntry.NodeKind != NodeKind.Directory);
    
    
    // private Dictionary<string, StatusEntry> _checkedItems = [];
    // private HashSet<string> _expandedItems = [];

    private TreeItemViewModel? _root;
    
    private readonly SingleTaskQueue _singleTaskQueue = new();

    /// <inheritdoc/>
    public LocalViewModel(ViewModelBase parent) : base(parent)
    {
        SelectedTreeItems.CollectionChanged += (sender, args) =>
        {
            NotifyProperties();
        };
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
    }


    private async Task Unlock()
    {
        if (!IsUnlockButtonEnable)
        {
            return;
        }


        // var unlockOptions = new UnlockOptions();
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
        if (!IsRevertButtonEnable)
        {
            return;
        }
        
        var revertOptions = new RevertOptions(
            SelectedTreeItems.Select(i => i.StatusEntry.Path).ToArray(),
            Depth.Infinity,
            [],
            false,
            false,
            true
        );

        var context = SendMessage(new OnGetContext()).Response;

        await context.Revert(revertOptions);

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

    partial void OnShowRootChanged(bool value)
    {
        if (_root is null)
        {
            return;
        }
        
        TreeItems.Clear();
        if (value)
        {
            TreeItems.Add(_root);
        }
        else
        {
            TreeItems.AddRange(_root.Children);
        }
    }

    private async Task RefreshItem(TreeItemViewModel item)
    {
        await item.RefreshChildren();
        foreach (var child in item.Children)
        {
            _ = Dispatcher.UIThread.InvokeAsync(async () =>
            {
                await RefreshItem(child);
            });
        }
    }

    [RelayCommand]
    private async Task Refresh()
    {
        await _singleTaskQueue.Run(async token =>
        {
            if (_root is null)
            {
                if (!IsLoading)
                {
                    token.ThrowIfCancellationRequested();
                    await LoadRoot();
                }
            }
            token.ThrowIfCancellationRequested();

            if (_root is not null)
            {
                await RefreshItem(_root);
            }
            CleanWeakTreeItems();
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
            if (_root is null && !IsLoading)
            { 
                token.ThrowIfCancellationRequested();
                await LoadRoot();
            }

            if (_root is null)
            {
                return;
            }
            token.ThrowIfCancellationRequested();
            await LoadAllChildren(_root);
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

            var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

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
                    _root = new TreeItemViewModel(this)
                    {
                        StatusEntry = entry,
                    };
                    
                    Receive(new OnCreatedItem(_root));
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

            _root?.Children.AddRange(children);
            _root?.HasLoaded = true;
            _root?.IsExpanded = true;
            OnShowRootChanged(ShowRoot);
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

    [RelayCommand]
    private async Task OnLoaded()
    {
        await _singleTaskQueue.Run(async _ => await LoadRoot());
    }

    public void Receive(OnCreatedItem message)
    {
        _weakTreeItems.Add(new WeakReference<TreeItemViewModel>(message.Value));
    }
}