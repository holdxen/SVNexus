using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Models;
using SVNexus.Utils;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesViewModel: ViewModelBase, IRecipient<Messages.OnSelectedItemChanged>, IRecipient<Messages.OnSelectedItemsChanged>, IRecipient<OnRefreshWorkingCopy>
{
    private readonly LimitedDictionary<string, DifferenceViewModel> _differenceViewModels = new()
    {
        Limit = 20,
    };

    [ObservableProperty] public partial DifferenceViewModel DifferenceViewModel { get; set; }

    private const int ListViewIndex = 0;
    private const int TreeViewIndex = 1;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsTreeView))]
    [NotifyPropertyChangedFor(nameof(IsListView))]
    public partial int SelectedViewIndex { get; set; } = ListViewIndex;

    public bool IsTreeView => SelectedViewIndex == TreeViewIndex;

    public bool IsListView => SelectedViewIndex == ListViewIndex;

    public ChangesListViewModel ChangesListViewModel { get; }

    public ChangesTreeViewModel ChangesTreeViewModel { get; }

    // [ObservableProperty] public partial LoadingOrErrorState EditorState { get; set; } = LoadingOrErrorState.MakeNone();
    //
    // [ObservableProperty] public partial Difference Difference { get; set; } = new();

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsRevertButtonEnable))]
    [NotifyPropertyChangedFor(nameof(IsLockButtonEnable))]
    [NotifyPropertyChangedFor(nameof(IsUnlockButtonEnable))]
    [NotifyPropertyChangedFor(nameof(IsCommitButtonEnable))]
    [NotifyPropertyChangedFor(nameof(IsOpenTerminalButtonEnable))]
    [NotifyPropertyChangedFor(nameof(IsInfoButtonEnable))]
    public partial List<StatusEntry> SelectedStatusEntries { get; set; } = [];

    public bool IsRevertButtonEnable => SelectedStatusEntries.Count > 0;
    
    public bool IsLockButtonEnable => SelectedStatusEntries.Count > 0 && SelectedStatusEntries.All(i => i is { NodeKind: NodeKind.File, Lock: null });
    
    public bool IsUnlockButtonEnable => SelectedStatusEntries.Count > 0 && SelectedStatusEntries.All(i => i.NodeKind != NodeKind.Directory && i.Lock is not null);

    public bool IsCommitButtonEnable => SelectedStatusEntries.Count > 0;
    
    public bool IsInfoButtonEnable => SelectedStatusEntries.Count == 1;
    
    
    public bool IsOpenTerminalButtonEnable
    {
        get
        {
            if (SelectedStatusEntries.Count != 1)
            {
                return false;
            }

            var item = SelectedStatusEntries.First();
            if (item.NodeKind == NodeKind.File)
            {
                return true;
            }

            return item.NodeStatus is not (WorkingCopyStatus.Missing or WorkingCopyStatus.Deleted);
        }
    }



    public ChangesViewModel(ViewModelBase parent): base(parent)
    {
        ChangesTreeViewModel = new ChangesTreeViewModel(this);
        ChangesListViewModel = new ChangesListViewModel(this);
        DifferenceViewModel = new DifferenceViewModel(this);
    }

    [RelayCommand]
    private void SwitchToListView()
    {
        SelectedViewIndex = ListViewIndex;
    }
    
    [RelayCommand]
    private void SwitchToTreeView()
    {
        SelectedViewIndex = TreeViewIndex;
    }


    [RelayCommand]
    private async Task Info()
    {
        var model = new InfoDialogModel(this)
        {
            Path = SelectedStatusEntries.First().Path,
            PegRevision = new Revision.Unspecified(),
            Revision =  new Revision.Unspecified(),
        };

        await model.LoadedCommand.ExecuteOrNothingAsync(null);

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

        var platform = IPlatform.Create();
        
        var item = SelectedStatusEntries.First();
        string path;
        if (item.NodeKind == NodeKind.Directory)
        {
            path = item.Path;
        }
        else
        {
            path = item.Path.GetDirectoryName() ?? platform.FileSystemRootPath;
        }


        await platform.OpenTerminal(path);
    }

    [RelayCommand]
    private async Task Lock()
    {
        var model = new LockDialogModel(this)
        {
            Targets = SelectedStatusEntries.Select(i => TargetItemViewModel.From(i, false, SendMessage(new OnGetWorkingCopyPath()))).ToList(),
        };
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowStandardAsync<LockDialog, LockDialogModel>(model, hostId, model.OverlayDialogOptions);
        if (model.Accept)
        {
            await StatusCommand.ExecuteOrNothingAsync(null);
        }
    }

    [RelayCommand]
    private async Task Unlock()
    {
        var model = new UnlockDialogModel(this)
        {
            // RelateTo = SendMessage(new OnGetWorkingCopyPath()),
            // TargetEntries = SelectedStatusEntries,
            Targets = SelectedStatusEntries.Select(i => TargetItemViewModel.From(i, false, SendMessage(new OnGetWorkingCopyPath()))).ToList()
        };

        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowStandardAsync<UnlockDialog, UnlockDialogModel>(model, hostId, model.OverlayDialogOptions);
        if (model.Accept)
        {
            await StatusCommand.ExecuteOrNothingAsync(null);
        }
    }
    
    [RelayCommand]
    private async Task Revert()
    {
        var model = new RevertDialogModel(this)
        {
            Targets = SelectedStatusEntries.Select(i => TargetItemViewModel.From(i, false, SendMessage(new OnGetWorkingCopyPath()))).ToList(),
        };


        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowStandardAsync<RevertDialog, RevertDialogModel>(model, hostId, model.OverlayDialogOptions);

        if (model.Accept)
        {
            await StatusCommand.ExecuteOrNothingAsync(null);
        }

    }

    
    [RelayCommand]
    public async Task Status()
    {
        try
        {

            var context = SendMessage(new OnGetContext()).Response;

            var hostId = SendMessage(new OnGetDialogHostId()).Response;
            var path = SendMessage(new OnGetWorkingCopyPath());

            var statusOptions = new StatusOptions(
                Path: path,
                Revision: new Revision.Working(),
                Depth: Depth.Infinity,
                GetAll: false,
                CheckOutOfDate: false,
                CheckWorkingCopy: false,
                NoIgnore: false,
                IgnoreExternals: true,
                DepthAsSticky: false, Changelist: null);


            var result = await context.Status(statusOptions);


            var lockedList = result.Entries.Where(e => e.WcIsLocked).ToList();
            if (lockedList.Count > 0)
            {
                var boxResult = await OverlayMessageBox.ShowAsync(
                    title: "Error",
                    hostId: hostId,
                    message: "Working copy is locked\nTry to cleanup now",
                    button: MessageBoxButton.YesNo);
                if (boxResult == MessageBoxResult.Yes)
                {
                    var cleanupOptions = new CleanupOptions(
                        Path: path,
                        BreakLocks: true,
                        FixRecordedTimestamps: true,
                        ClearDavCache: true,
                        VacuumPristines: true,
                        IncludeExternals: false);

                    await context.Cleanup(cleanupOptions);

                    await Status();
                }
                else
                {
                    SendMessage(new OnRemoveTabModel());
                    return;
                }
            }

            var incompleteList = result.Entries.Where(e => e.NodeStatus == WorkingCopyStatus.Incomplete).ToList();
            if (incompleteList.Count > 0)
            {
                var boxResult = await OverlayMessageBox.ShowAsync(
                    title: "Error",
                    hostId: hostId,
                    message: "Working copy is incomplete\nTry to update now",
                    button: MessageBoxButton.YesNo);
                if (boxResult == MessageBoxResult.Yes)
                {
                    // todo: handle update
                    await Status();
                }
                else
                {
                    SendMessage(new OnRemoveTabModel());
                    return;
                }
            }

            _differenceViewModels.Clear();
            Update(result.Entries);
        }
        catch (System.Exception e)
        {
            var handled = e.Handle(svnExceptionHandler: error =>
            {
                var errorNumber = new SvnErrnoConstants();
                if (!errorNumber.IsWcNotDirectory(error.Code)) return false;
                // Manager.Default.Send(new OnNotWorkingCopy(_workingCopyViewService.WorkingCopyPath), _tabService.Token);
                SendMessage(new OnNotWorkingCopy());
                return true;
            });
            if (!handled)
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = $"Failed to status:\n{e.HumanReadableMessage}",
                    Type =  NotificationType.Error
                }, Manager.MainWindowToken);
            }
        }
    }

    private void Update(StatusEntry[] statusEntries)
    {
        ChangesListViewModel.Update(statusEntries);
        
        ChangesTreeViewModel.Update(statusEntries);
    }


    [RelayCommand]
    private async Task Commit()
    {
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId()).Response;
        
        var items = IsTreeView ? ChangesTreeViewModel.SelectedItems.Select(i => i.StatusEntry!).ToArray() : ChangesListViewModel.SelectedItems.Select(i => i.Entry).ToArray();

        if (items.Length == 0)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = "No files or folders checked.",
                Type =  NotificationType.Warning,
            }, Manager.MainWindowToken);
            return;
        }

        var model = new CommitDialogModel(this)
        {
            Targets = items,
            RelateTo = SendMessage(new OnGetWorkingCopyPath())
        };


        await OverlayDialog.ShowStandardAsync<CommitDialog, CommitDialogModel>(model, hostId, model.OverlayDialogOptions);

        if (model.Accept)
        {
            await StatusCommand.ExecuteOrNothingAsync(null);
        }

    }

    public void Receive(Messages.OnSelectedItemChanged message)
    {
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            if (message.Value == null)
            {
                DifferenceViewModel = new DifferenceViewModel(this);
                return;
            }
            
            if (_differenceViewModels.TryGetValue(message.Value.Path, out var difference))
            {
                DifferenceViewModel = difference;
                return;
            }
            
            DifferenceViewModel = new DifferenceViewModel(this);
            
            _differenceViewModels.Add(message.Value.Path, DifferenceViewModel);
            
            await DifferenceViewModel.CompareWorkingCopyEntryCommand.ExecuteOrNothingAsync(message.Value);
        });

    }

    public void Receive(OnRefreshWorkingCopy message)
    {
        Dispatcher.UIThread.InvokeAsync(async () => await Status());
    }

    [RelayCommand]
    private async Task OnShow()
    {
        await Status();
    }

    public void Receive(Messages.OnSelectedItemsChanged message)
    {
        // 当界面显示的时候或者直接调用status进行更新的时候，会导致tree和list都重新计算selected item，
        // 计算完成后都把结果会发送到这里，导致list的结果被tree覆盖，所以要判断消息是否是当前正在显示的view发的，否则直接忽略
        if ((IsTreeView && Sender == ChangesTreeViewModel) || (IsListView && Sender == ChangesListViewModel))
        {
            SelectedStatusEntries = message.Value;
        }
    }
}