using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Models;
using SVNexus.Utils;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesViewModel: ViewModelBase, IRecipient<OnSelectedItemChanged>, IRecipient<OnRefreshWorkingCopy>
{
    // private readonly IServiceProvider _serviceProvider;

    private class DifferencesCat
    {
        public required CatResult Original { get; set; }
        public required CatResult Modified { get; set; }
    }

    private readonly LimitedDictionary<string, DifferencesCat?> _differences = new()
    {
        Limit = 20
    };

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


    [ObservableProperty]
    public partial string CommitMessage { get; set; } = string.Empty;

    //
    // [ObservableProperty]
    // public partial string WorkingCopyPath { get; set; } = string.Empty;
    

    [ObservableProperty] public partial LoadingOrErrorState EditorState { get; set; } = LoadingOrErrorState.MakeNone();

    [ObservableProperty] public partial Difference Difference { get; set; } = new();
    
    public string? SelectedItem { get; set; }
    
    public static Type DepthType => typeof(Depth);
    
    
    [ObservableProperty]
    public partial Depth Depth { get; set; } = Depth.Infinity;

    [ObservableProperty] public partial bool CommitMissingAsDelete { get; set; } = true;

    [ObservableProperty] public partial bool AddUnversionedBeforeCommit { get; set; } = true;

    
    [ObservableProperty]
    public partial bool IsWcRoot { get; set; }
    
    [ObservableProperty]
    public partial bool Upgradable { get; set; }

    // private readonly Services.ITabService _tabService;
    // private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    // private readonly Services.TypeService _typeService;
    public ChangesViewModel(ViewModelBase parent): base(parent)
    {
        // _serviceProvider = serviceProvider;
        // _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
        // _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
        // _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
        
        // ChangesTreeViewModel = serviceProvider.GetRequiredService<ChangesTreeViewModel>();
        // ChangesListViewModel = serviceProvider.GetRequiredService<ChangesListViewModel>();
        ChangesTreeViewModel = new ChangesTreeViewModel(this);
        ChangesListViewModel = new ChangesListViewModel(this);
        DifferenceViewModel = new DifferenceViewModel(this);
        
        // Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
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

    // [RelayCommand]
    // private async Task OnLoaded()
    // {
    //     await Status();
    // }



    [RelayCommand]
    private async Task Relocate()
    {
        
    }

    [RelayCommand]
    private async Task DiffAndExport()
    {
        
    }
    
    [RelayCommand]
    private async Task ApplyPatch()
    {
        
    }


    // private async Task CheckWorkingCopyVersion()
    // {
    //     var path = SendMessage(new OnGetWorkingCopyPath());
    //     var context = SendMessage(new OnGetContext()).Response;
    //     using var wc = context.WorkingCopyContext();
    //     
    //     var current = await wc.WcVersion(path);
    //     
    //     var recommended = await context.DefaultWcVersion();
    //
    //     Upgradable = recommended.Compare(current) > 0;
    // }
    

    // private async Task CheckWorkingCopyRoot()
    // {
    //     try
    //     {
    //         var path = SendMessage(new OnGetWorkingCopyPath());
    //         using var context = Engine.Engine.Instance.SimpleContext(SendMessage(new OnGetDialogHostId()).Response);
    //         using var wc = context.WorkingCopyContext();
    //         var result = await wc.CheckRoot(path);
    //         IsWcRoot = result.IsWcRoot;
    //     }
    //     catch (System.Exception e)
    //     {
    //         IsWcRoot = false;
    //     }
    //     
    // }


    [RelayCommand]
    private async Task Revert()
    {
        Dictionary<string, StatusEntry>? items;
        if (IsTreeView)
        {
            items = ChangesTreeViewModel.CheckedItems;
        } 
        else if (IsListView)
        {
            items = ChangesListViewModel.CheckedItems;
        }
        else
        {
            return;
        }

        if (items.Count == 0)
        {
            return;
        }

        List<string> versioned = [];
        List<string> unversioned = [];

        foreach (var item in items)
        {
            if (item.Value.NodeStatus == WorkingCopyStatus.Unversioned)
            {
                unversioned.Add(item.Key);
            }
            else
            {
                versioned.Add(item.Key);
            }
        }


        if (versioned.Count == 0)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = "No items will be reverted",
                Type = NotificationType.Warning
            }, Manager.MainWindowToken);
            return;
        }
        var path = SendMessage(new OnGetWorkingCopyPath());
        
        var message = unversioned.Count > 0 ?
            $"The following files will be reverted:\n{string.Join(Environment.NewLine, versioned.Select(e => e.TrimStartString(path).TrimStartPathSeparatorChar()))}\n\nThe following files will be skipped:\n{string.Join(Environment.NewLine, unversioned.Select(e => e.TrimStartString(path).TrimStartPathSeparatorChar()))}" 
            : $"The following files will be reverted:\n{string.Join(Environment.NewLine, versioned.Select(e => e.TrimStartString(path).TrimStartPathSeparatorChar()))}";
        
        
        
        
        var hostId = SendMessage(new OnGetDialogHostId()).Response;
        var result = await MessageBox.ShowOverlayAsync(message,
            "Warning",
            hostId,
            MessageBoxIcon.Warning,
            MessageBoxButton.YesNo
            );
        if (result != MessageBoxResult.Yes)
        {
            return;
        }
        
        

        var revertOptions = new RevertOptions(
            Paths: versioned.ToArray(), 
            Depth: Depth.Empty, 
            Changelists: [], 
            ClearChangelists: false, 
            MetadataOnly: false, 
            AddedKeepLocal: true
            );
        

        // using var context = Engine.Engine.Instance.SimpleContext(hostId);

        var context = SendMessage(new OnGetContext()).Response;

        try
        {
            await context.Revert(revertOptions);
            await Status();
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
                    await Update();
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
    //
    // [RelayCommand]
    // public async Task Status()
    // {
    //     // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
    //     // var hostId = SendMessage(new OnGetDialogHostId()).Response;
    //
    //     // using var context = Engine.Engine.Instance.SimpleContext(hostId);
    //     
    //     await Status(context);
    // }
    //
    [RelayCommand]
    private async Task Update()
    {
        try
        {
            var context = SendMessage(new OnGetContext()).Response;
            var path = SendMessage(new OnGetWorkingCopyPath());
            
            var opts = new UpdateOptions(
                Paths: [path], 
                Revision: new Revision.Head(), 
                Depth: Depth.Infinity, 
                DepthIsSticky: false, 
                IgnoreExternals: false, 
                AllowUnverObstructions: true, 
                AddsAsModification: false, 
                MakeParents: true);
            
            await context.Update(opts);

            Manager.Default.Send(new OnShowToast()
            {
                Content = "Update completed",
                Type = NotificationType.Success
            }, Manager.MainWindowToken);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to update:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error,
            }, Manager.MainWindowToken);
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
        
        var items = IsTreeView ? ChangesTreeViewModel.CheckedItems : ChangesListViewModel.CheckedItems;

        if (items.Count == 0)
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
            Targets = items.Values.ToArray(),
            RelateTo = SendMessage(new OnGetWorkingCopyPath())
        };

        var dialogOptions = new OverlayDialogOptions()
        {
            IsCloseButtonVisible = false,
            Title = "Commit",
            Buttons = DialogButton.None,
            CanDragMove = true,
            CanResize = true,
            StyleClass = "Fixed"
        };

        await OverlayDialog.ShowModal<CommitDialog, CommitDialogModel>(model, hostId, dialogOptions);

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

    public void Receive(OnSelectedItemChanged message)
    {
        if (message.Value is not StatusEntry statusEntry) return;
        SelectedItem = statusEntry.Path;
        
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            await DifferenceViewModel.CompareWorkingCopyEntry(statusEntry);
        });

        
        // TryCompareStatusEntry(statusEntry);
    }


    [RelayCommand]
    private void TryCompareStatusEntry(StatusEntry statusEntry)
    {
        if (statusEntry.NodeKind is NodeKind.Directory)
        {
            Difference = new Difference();
            EditorState = LoadingOrErrorState.MakeNone();
            return;
        }
        var contains = _differences.TryGetValue(statusEntry.Path, out var difference);
        if (contains)
        {
            if (difference is null) return;
        }
        
        Logger.Info($"Selected Entry: {statusEntry}");
            
        _differences[statusEntry.Path] = null;
        EditorState = LoadingOrErrorState.MakeLoading();

        Dispatcher.UIThread.InvokeAsync(async () =>
        {

            var success = false;
            // AsyncContext? context = null;
            try
            {
                // context = Engine.Engine.Instance.SimpleContext(SendMessage(new OnGetDialogHostId()).Response);
                var context = SendMessage(new OnGetContext()).Response;
                
                // var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                //     Revision: new Revision.Working(), ExpandKeywords: true);
                // var resultModified = await context.Cat(catOptions);
                //
                //
                // catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                //     Revision: new Revision.Base(), ExpandKeywords: true);
                // var resultOriginal = await context.Cat(catOptions);

                CatResult resultOriginal;

                Func<Task<CatResult>> catModified;
                Func<Task<CatResult>> catOriginal;

                if (statusEntry.NodeStatus is WorkingCopyStatus.Missing or WorkingCopyStatus.Deleted)
                {
                    catModified = () => Task.FromResult(new CatResult([], []));
                    catOriginal = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Base(),
                            Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
                        return context.Cat(catOptions);
                    };
                }
                else if (statusEntry.NodeStatus is WorkingCopyStatus.Unversioned or WorkingCopyStatus.Added)
                {
                    catModified = async () =>
                    {
                        var content = await File.ReadAllBytesAsync(statusEntry.Path);
                        return new  CatResult(content, []);
                    };
                    catOriginal = () => Task.FromResult(new  CatResult([], []));
                    
                }
                else
                {
                    catModified = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                            Revision: new Revision.Working(), ExpandKeywords: true, GetProperties: false);
                        return context.Cat(catOptions);
                    };
                    catOriginal = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                            Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
                        return context.Cat(catOptions);
                    };
                }

                var resultModified = await catModified();
                if (difference is null)
                {
                    resultOriginal = await catOriginal();
                }
                else
                {
                    resultOriginal = difference.Original;
                }
                
                
                // if (difference is null)
                // {
                //     if (statusEntry.NodeStatus is NodeStatus.Unversioned or NodeStatus.Added)
                //     {
                //         var content = await File.ReadAllBytesAsync(statusEntry.Path);
                //         resultModified = new CatResult(content, []);
                //         resultOriginal = new CatResult([], []);
                //     }
                //     else if (statusEntry.NodeStatus is NodeStatus.Missing or NodeStatus.Deleted)
                //     {
                //         var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                //             Revision: new Revision.Base(), ExpandKeywords: true);
                //         Log.Info($"Cat original: {statusEntry.Path}");
                //         resultOriginal = await context.Cat(catOptions);
                //         Log.Info("Finished cat");
                //         resultModified = new CatResult([], []);
                //     }
                //     else
                //     {
                //         var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                //             Revision: new Revision.Working(), ExpandKeywords: true);
                //         resultModified = await context.Cat(catOptions);
                //     
                //         catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                //             Revision: new Revision.Base(), ExpandKeywords: true);
                //         resultOriginal = await context.Cat(catOptions);
                //     }
                // }
                // else
                // {
                //     var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                //         Revision: new Revision.Working(), ExpandKeywords: true);
                //     resultModified = await context.Cat(catOptions);
                //     
                //     resultOriginal = difference.Original;
                // }
                
                var modified = Encoding.UTF8.GetString(resultModified.Content).Split("\n").Select(e =>
                    new DifferenceLine()
                    {
                        Content = e,
                        DifferenceKind = DifferenceLine.Kind.Unchanged
                    }).ToList();
                if (modified.Count > 0 && string.IsNullOrEmpty(modified.Last().Content))
                {
                    modified.RemoveAt(modified.Count - 1);
                }

                var original = Encoding.UTF8.GetString(resultOriginal.Content).Split("\n")
                    .Select(e => new DifferenceLine()
                    {
                        Content = e,
                        DifferenceKind = DifferenceLine.Kind.Unchanged
                    }).ToList();
                
                if (original.Count > 0 && string.IsNullOrEmpty(original.Last().Content))
                {
                    original.RemoveAt(original.Count - 1);
                }


                var diffOptions =
                    new DifferenceOptions(Original: resultOriginal.Content, Modified: resultModified.Content,
                        Options: new DifferenceFileOptions(DiffFileIgnoreSpace.None, false, false, 0));

                var changes = diffOptions.Exec().Modified;

                // changes = [
                //     new TextChange(Original: null, Modified: new TextPosition(3, 1)),
                //     new TextChange(Original: null, Modified: new TextPosition(5, 1))
                // ];

                foreach (var change in changes)
                {
                    // Console.WriteLine("Go change: {0}", change);
                    if (change.Original.Len == 0 && change.Modified.Len > 0) // added
                    {
                        original.InsertRange(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add]),
                            Enumerable.Repeat(new DifferenceLine()
                            {
                                Content = "",
                                DifferenceKind = DifferenceLine.Kind.Add
                            }, (int)change.Modified.Len));


                        foreach (var differenceLine in modified
                                     .Skip(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]))
                                     .Take((int)change.Modified.Len))
                        {
                            differenceLine.DifferenceKind = DifferenceLine.Kind.Added;
                        }
                    }
                    else if (change.Original.Len > 0 && change.Modified.Len == 0) // remove
                    {
                        foreach (var differenceLine in original
                                     .Skip(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add])).Take((int)change.Original.Len))
                        {
                            differenceLine.DifferenceKind = DifferenceLine.Kind.Remove;
                        }

                        modified.InsertRange(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]),
                            Enumerable.Repeat(new DifferenceLine()
                            {
                                Content = "",
                                DifferenceKind = DifferenceLine.Kind.Removed
                            }, (int)change.Original.Len));

                    }
                    else if (change.Original.Len > 0 && change.Modified.Len > 0)
                    {
                        {
                            var pos = (int)change.Original.Pos;
                            var len = (int)change.Original.Len;
                            foreach (var differenceLine in original
                                         .Skip(original.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Add]))
                                         .Take(len))
                            {
                                differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
                            }
                        }
                        {
                            var pos = (int)change.Modified.Pos;
                            var len = (int)change.Modified.Len;
                            foreach (var differenceLine in modified
                                         .Skip(modified.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Removed]))
                                         .Take(len))
                            {
                                differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
                            }
                        }
                    }
                }
                var info = new Difference()
                {
                    Original = original,
                    Modified = modified,
                };


                _differences[statusEntry.Path] = new DifferencesCat()
                {
                    Original = resultOriginal,
                    Modified = resultModified,
                };

                if (SelectedItem == statusEntry.Path)
                {
                    Difference = info;
                    EditorState = LoadingOrErrorState.MakeNone();
                }
                
                // Console.WriteLine("Diff success");

                success = true;

            }
            catch (System.Exception e)
            {
                Console.WriteLine(e);
                EditorState = new LoadingOrErrorState.Error()
                {
                    ErrorMessage = $"Failed to load content: {e.HumanReadableMessage}",
                    RetryCommand = TryCompareStatusEntryCommand,
                    RetryCommandParameter = statusEntry
                };
                success = false;
            }
            finally
            {
                if (!success)
                {
                    _differences.Remove(statusEntry.Path);
                }
                else if (SelectedItem == statusEntry.Path)
                {
                    EditorState = LoadingOrErrorState.MakeNone();
                }
            }
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
}