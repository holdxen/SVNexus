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
    // private class DifferencesCat
    // {
    //     public required CatResult Original { get; set; }
    //     public required CatResult Modified { get; set; }
    // }

    // private readonly LimitedDictionary<string, DifferencesCat?> _differences = new()
    // {
    //     Limit = 20
    // };

    private readonly LimitedDictionary<string, DifferenceViewModel> _differenceViewModels = new()
    {
        Limit = 20,
    };

    [ObservableProperty] public partial DifferenceViewModel DifferenceViewModel { get; set; }

    public const int ListViewIndex = 0;
    public const int TreeViewIndex = 1;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsTreeView))]
    [NotifyPropertyChangedFor(nameof(IsListView))]
    public partial int SelectedViewIndex { get; set; } = ListViewIndex;

    public bool IsTreeView => SelectedViewIndex == TreeViewIndex;

    public bool IsListView => SelectedViewIndex == ListViewIndex;

    public ChangesListViewModel ChangesListViewModel { get; }

    public ChangesTreeViewModel ChangesTreeViewModel { get; }

    [ObservableProperty] public partial LoadingOrErrorState EditorState { get; set; } = LoadingOrErrorState.MakeNone();

    [ObservableProperty] public partial Difference Difference { get; set; } = new();
    
    public string? SelectedItem { get; set; }

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
            await OverlayDialog.ShowModal<InfoDialog, InfoDialogModel>(model, hostId, model.OverlayDialogOptions);
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
        var model = new RevertDialogModel(this)
        {
            RelateTo = SendMessage(new OnGetWorkingCopyPath()),
            TargetEntries = SelectedStatusEntries,
        };
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<RevertDialog, RevertDialogModel>(model, hostId, model.OverlayDialogOptions);
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
            RelateTo = SendMessage(new OnGetWorkingCopyPath()),
            TargetEntries = SelectedStatusEntries,
        };

        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<UnlockDialog, UnlockDialogModel>(model, hostId, model.OverlayDialogOptions);
        if (model.Accept)
        {
            await StatusCommand.ExecuteOrNothingAsync(null);
        }
    }
    
    [RelayCommand]
    private async Task Revert()
    {
        // Dictionary<string, StatusEntry> items = [];
        //
        // if (items.Count == 0)
        // {
        //     return;
        // }
        //
        // List<string> versioned = [];
        // List<string> unversioned = [];
        //
        // foreach (var item in items)
        // {
        //     if (item.Value.NodeStatus == WorkingCopyStatus.Unversioned)
        //     {
        //         unversioned.Add(item.Key);
        //     }
        //     else
        //     {
        //         versioned.Add(item.Key);
        //     }
        // }
        //
        //
        // if (versioned.Count == 0)
        // {
        //     Manager.Default.Send(new OnShowToast()
        //     {
        //         Content = "No items will be reverted",
        //         Type = NotificationType.Warning
        //     }, Manager.MainWindowToken);
        //     return;
        // }
        // var path = SendMessage(new OnGetWorkingCopyPath());
        //
        // var message = unversioned.Count > 0 ?
        //     $"The following files will be reverted:\n{string.Join(Environment.NewLine, versioned.Select(e => e.TrimStartString(path).TrimStartPathSeparatorChar()))}\n\nThe following files will be skipped:\n{string.Join(Environment.NewLine, unversioned.Select(e => e.TrimStartString(path).TrimStartPathSeparatorChar()))}" 
        //     : $"The following files will be reverted:\n{string.Join(Environment.NewLine, versioned.Select(e => e.TrimStartString(path).TrimStartPathSeparatorChar()))}";
        //
        //
        //
        //
        // var hostId = SendMessage(new OnGetDialogHostId()).Response;
        // var result = await MessageBox.ShowOverlayAsync(message,
        //     "Warning",
        //     hostId,
        //     MessageBoxIcon.Warning,
        //     MessageBoxButton.YesNo
        //     );
        // if (result != MessageBoxResult.Yes)
        // {
        //     return;
        // }
        //
        //
        //
        // var revertOptions = new RevertOptions(
        //     Paths: versioned.ToArray(), 
        //     Depth: Depth.Empty, 
        //     Changelists: [], 
        //     ClearChangelists: false, 
        //     MetadataOnly: false, 
        //     AddedKeepLocal: true
        //     );
        //
        //
        // // using var context = Engine.Engine.Instance.SimpleContext(hostId);
        //
        // var context = SendMessage(new OnGetContext()).Response;
        //
        // try
        // {
        //     await context.Revert(revertOptions);
        //     await Status();
        // }
        // catch (System.Exception e)
        // {
        //     Manager.Default.Send(new OnShowToast()
        //     {
        //         Content = $"Failed to revert:\n{e.HumanReadableMessage}",
        //         Type = NotificationType.Error
        //     }, Manager.MainWindowToken);
        // }
        //
        //
        //
        //

        var model = new RevertDialogModel(this)
        {
            TargetEntries = SelectedStatusEntries,
            RelateTo = SendMessage(new OnGetWorkingCopyPath()),
        };


        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<RevertDialog, RevertDialogModel>(model, hostId, model.OverlayDialogOptions);

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
                var boxResult = await MessageBox.ShowOverlayAsync(
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
                var boxResult = await MessageBox.ShowOverlayAsync(
                    title: "Error",
                    hostId: hostId,
                    message: "Working copy is incomplete\nTry to cleanup now",
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


        await OverlayDialog.ShowModal<CommitDialog, CommitDialogModel>(model, hostId, model.OverlayDialogOptions);

        if (model.Accept)
        {
            await StatusCommand.ExecuteOrNothingAsync(null);
        }

    }

//     [RelayCommand]
//     private async Task Commit()
//     {
//         try
//         {
//             if (string.IsNullOrEmpty(CommitMessage))
//             {
//                 Manager.Default.Send(new OnShowToast()
//                 {
//                     Content = "Empty commit message is not allowed",
//                     Type = NotificationType.Warning
//                 }, Manager.MainWindowToken);
//                 return;
//             }
//             var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;
//
//             var context = Engine.Engine.Instance.SimpleContext(hostId);
//
//             List<string> unversioned = [];
//         
//             List<string> missing = [];
//         
//             List<string> others = [];
//
//
//             var items = IsTreeView ? ChangesTreeViewModel.CheckedItems : ChangesListViewModel.CheckedItems;
//
//             foreach (var item in items)
//             {
//                 switch (item.Value.NodeStatus)
//                 {
//                     case NodeStatus.Unversioned:
//                         unversioned.Add(item.Key);
//                         break;
//                     case NodeStatus.Missing:
//                         missing.Add(item.Key);
//                         break;
//                     case NodeStatus.None:
//                     case NodeStatus.Normal:
//                     case NodeStatus.Added:
//                     case NodeStatus.Deleted:
//                     case NodeStatus.Replaced:
//                     case NodeStatus.Modified:
//                     case NodeStatus.Merged:
//                     case NodeStatus.Conflicted:
//                     case NodeStatus.Ignored:
//                     case NodeStatus.Obstructed:
//                     case NodeStatus.External:
//                     case NodeStatus.Incomplete:
//                     default:
//                         others.Add(item.Key);
//                         break;
//                 }
//             }
//
//             if (missing.Count > 0 && CommitMissingAsDelete)
//             {
//                 await context.Delete(new DeleteOptions(missing.ToArray(), false, false, []));
//             
//                 others.AddRange(missing);
//             }
//
//             if (unversioned.Count > 0 && AddUnversionedBeforeCommit)
//             {
//                 foreach (var item in unversioned)
//                 {
//                     await context.Add(new AddOptions(item, Depth.Empty, false, false, false, false));
//                 }
//             
//                 others.AddRange(unversioned);
//             }
//
//             if (others.Count == 0)
//             {
//                 return;
//             }
//
//
//             var commitOptions = new CommitOptions(
//                 others.ToArray(),
//                 Depth: Depth,
//                 KeepLocks: true,
//                 KeepChangelist: false, 
//                 CommitAsOperations: true, 
//                 IncludeFileExternals: true, 
//                 IncludeDirExternals: true,
//                 Changelists: [], 
//                 RevisionPropertyTable: new Dictionary<string, string>(), 
//                 CommitMessage: CommitMessage);
//         
//             await context.Commit(commitOptions);
//             
//             await Status(context);
//
//             Manager.Default.Send(new OnShowToast()
//             {
//                 Content = "Commit successfully",
//                 Type =  NotificationType.Success,
//             }, Manager.MainWindowToken);
//         }
//         catch (System.Exception e)
//         {
//             Manager.Default.Send(new OnShowToast()
//             {
//                 Content = $"Failed to commit: {e.HumanReadableMessage}",
//                 Type =  NotificationType.Error,
//             }, Manager.MainWindowToken);
//         }
//
//     }

    public void Receive(Messages.OnSelectedItemChanged message)
    {
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            if (message.Value == null)
            {
                DifferenceViewModel = new DifferenceViewModel(this);
                SelectedItem = null;
                return;
            }

            SelectedItem = message.Value.Path;
            
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


    // [RelayCommand]
    // private void TryCompareStatusEntry(StatusEntry statusEntry)
    // {
    //     if (statusEntry.NodeKind is NodeKind.Directory)
    //     {
    //         Difference = new Difference();
    //         EditorState = LoadingOrErrorState.MakeNone();
    //         return;
    //     }
    //     var contains = _differences.TryGetValue(statusEntry.Path, out var difference);
    //     if (contains)
    //     {
    //         if (difference is null) return;
    //     }
    //     
    //     Logger.Info($"Selected Entry: {statusEntry}");
    //         
    //     _differences[statusEntry.Path] = null;
    //     EditorState = LoadingOrErrorState.MakeLoading();
    //
    //     Dispatcher.UIThread.InvokeAsync(async () =>
    //     {
    //
    //         var success = false;
    //         // AsyncContext? context = null;
    //         try
    //         {
    //             // context = Engine.Engine.Instance.SimpleContext(SendMessage(new OnGetDialogHostId()).Response);
    //             var context = SendMessage(new OnGetContext()).Response;
    //             
    //             // var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //             //     Revision: new Revision.Working(), ExpandKeywords: true);
    //             // var resultModified = await context.Cat(catOptions);
    //             //
    //             //
    //             // catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //             //     Revision: new Revision.Base(), ExpandKeywords: true);
    //             // var resultOriginal = await context.Cat(catOptions);
    //
    //             CatResult resultOriginal;
    //
    //             Func<Task<CatResult>> catModified;
    //             Func<Task<CatResult>> catOriginal;
    //
    //             if (statusEntry.NodeStatus is WorkingCopyStatus.Missing or WorkingCopyStatus.Deleted)
    //             {
    //                 catModified = () => Task.FromResult(new CatResult([], []));
    //                 catOriginal = () =>
    //                 {
    //                     var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Base(),
    //                         Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
    //                     return context.Cat(catOptions);
    //                 };
    //             }
    //             else if (statusEntry.NodeStatus is WorkingCopyStatus.Unversioned or WorkingCopyStatus.Added)
    //             {
    //                 catModified = async () =>
    //                 {
    //                     var content = await File.ReadAllBytesAsync(statusEntry.Path);
    //                     return new  CatResult(content, []);
    //                 };
    //                 catOriginal = () => Task.FromResult(new  CatResult([], []));
    //                 
    //             }
    //             else
    //             {
    //                 catModified = () =>
    //                 {
    //                     var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //                         Revision: new Revision.Working(), ExpandKeywords: true, GetProperties: false);
    //                     return context.Cat(catOptions);
    //                 };
    //                 catOriginal = () =>
    //                 {
    //                     var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //                         Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
    //                     return context.Cat(catOptions);
    //                 };
    //             }
    //
    //             var resultModified = await catModified();
    //             if (difference is null)
    //             {
    //                 resultOriginal = await catOriginal();
    //             }
    //             else
    //             {
    //                 resultOriginal = difference.Original;
    //             }
    //             
    //             
    //             // if (difference is null)
    //             // {
    //             //     if (statusEntry.NodeStatus is NodeStatus.Unversioned or NodeStatus.Added)
    //             //     {
    //             //         var content = await File.ReadAllBytesAsync(statusEntry.Path);
    //             //         resultModified = new CatResult(content, []);
    //             //         resultOriginal = new CatResult([], []);
    //             //     }
    //             //     else if (statusEntry.NodeStatus is NodeStatus.Missing or NodeStatus.Deleted)
    //             //     {
    //             //         var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //             //             Revision: new Revision.Base(), ExpandKeywords: true);
    //             //         Log.Info($"Cat original: {statusEntry.Path}");
    //             //         resultOriginal = await context.Cat(catOptions);
    //             //         Log.Info("Finished cat");
    //             //         resultModified = new CatResult([], []);
    //             //     }
    //             //     else
    //             //     {
    //             //         var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //             //             Revision: new Revision.Working(), ExpandKeywords: true);
    //             //         resultModified = await context.Cat(catOptions);
    //             //     
    //             //         catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //             //             Revision: new Revision.Base(), ExpandKeywords: true);
    //             //         resultOriginal = await context.Cat(catOptions);
    //             //     }
    //             // }
    //             // else
    //             // {
    //             //     var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
    //             //         Revision: new Revision.Working(), ExpandKeywords: true);
    //             //     resultModified = await context.Cat(catOptions);
    //             //     
    //             //     resultOriginal = difference.Original;
    //             // }
    //             
    //             var modified = Encoding.UTF8.GetString(resultModified.Content).Split("\n").Select(e =>
    //                 new DifferenceLine()
    //                 {
    //                     Content = e,
    //                     DifferenceKind = DifferenceLine.Kind.Unchanged
    //                 }).ToList();
    //             if (modified.Count > 0 && string.IsNullOrEmpty(modified.Last().Content))
    //             {
    //                 modified.RemoveAt(modified.Count - 1);
    //             }
    //
    //             var original = Encoding.UTF8.GetString(resultOriginal.Content).Split("\n")
    //                 .Select(e => new DifferenceLine()
    //                 {
    //                     Content = e,
    //                     DifferenceKind = DifferenceLine.Kind.Unchanged
    //                 }).ToList();
    //             
    //             if (original.Count > 0 && string.IsNullOrEmpty(original.Last().Content))
    //             {
    //                 original.RemoveAt(original.Count - 1);
    //             }
    //
    //
    //             var diffOptions =
    //                 new DifferenceOptions(Original: resultOriginal.Content, Modified: resultModified.Content,
    //                     Options: new DifferenceFileOptions(DiffFileIgnoreSpace.None, false, false, 0));
    //
    //             var changes = diffOptions.Exec().Modified;
    //
    //             // changes = [
    //             //     new TextChange(Original: null, Modified: new TextPosition(3, 1)),
    //             //     new TextChange(Original: null, Modified: new TextPosition(5, 1))
    //             // ];
    //
    //             foreach (var change in changes)
    //             {
    //                 // Console.WriteLine("Go change: {0}", change);
    //                 if (change.Original.Len == 0 && change.Modified.Len > 0) // added
    //                 {
    //                     original.InsertRange(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add]),
    //                         Enumerable.Repeat(new DifferenceLine()
    //                         {
    //                             Content = "",
    //                             DifferenceKind = DifferenceLine.Kind.Add
    //                         }, (int)change.Modified.Len));
    //
    //
    //                     foreach (var differenceLine in modified
    //                                  .Skip(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]))
    //                                  .Take((int)change.Modified.Len))
    //                     {
    //                         differenceLine.DifferenceKind = DifferenceLine.Kind.Added;
    //                     }
    //                 }
    //                 else if (change.Original.Len > 0 && change.Modified.Len == 0) // remove
    //                 {
    //                     foreach (var differenceLine in original
    //                                  .Skip(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add])).Take((int)change.Original.Len))
    //                     {
    //                         differenceLine.DifferenceKind = DifferenceLine.Kind.Remove;
    //                     }
    //
    //                     modified.InsertRange(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]),
    //                         Enumerable.Repeat(new DifferenceLine()
    //                         {
    //                             Content = "",
    //                             DifferenceKind = DifferenceLine.Kind.Removed
    //                         }, (int)change.Original.Len));
    //
    //                 }
    //                 else if (change.Original.Len > 0 && change.Modified.Len > 0)
    //                 {
    //                     {
    //                         var pos = (int)change.Original.Pos;
    //                         var len = (int)change.Original.Len;
    //                         foreach (var differenceLine in original
    //                                      .Skip(original.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Add]))
    //                                      .Take(len))
    //                         {
    //                             differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
    //                         }
    //                     }
    //                     {
    //                         var pos = (int)change.Modified.Pos;
    //                         var len = (int)change.Modified.Len;
    //                         foreach (var differenceLine in modified
    //                                      .Skip(modified.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Removed]))
    //                                      .Take(len))
    //                         {
    //                             differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
    //                         }
    //                     }
    //                 }
    //             }
    //             var info = new Difference()
    //             {
    //                 Original = original,
    //                 Modified = modified,
    //             };
    //
    //
    //             _differences[statusEntry.Path] = new DifferencesCat()
    //             {
    //                 Original = resultOriginal,
    //                 Modified = resultModified,
    //             };
    //
    //             if (SelectedItem == statusEntry.Path)
    //             {
    //                 Difference = info;
    //                 EditorState = LoadingOrErrorState.MakeNone();
    //             }
    //             
    //             // Console.WriteLine("Diff success");
    //
    //             success = true;
    //
    //         }
    //         catch (System.Exception e)
    //         {
    //             Console.WriteLine(e);
    //             EditorState = new LoadingOrErrorState.Error()
    //             {
    //                 ErrorMessage = $"Failed to load content: {e.HumanReadableMessage}",
    //                 RetryCommand = TryCompareStatusEntryCommand,
    //                 RetryCommandParameter = statusEntry
    //             };
    //             success = false;
    //         }
    //         finally
    //         {
    //             if (!success)
    //             {
    //                 _differences.Remove(statusEntry.Path);
    //             }
    //             else if (SelectedItem == statusEntry.Path)
    //             {
    //                 EditorState = LoadingOrErrorState.MakeNone();
    //             }
    //         }
    //     });
    //     
    // }
    //
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
        SelectedStatusEntries = message.Value;
    }
}