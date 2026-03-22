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
using SVNexus.Messages;
using SVNexus.Utils;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalViewModel: ViewModelBase, IRecipient<OnSelectedItemChanged>, IRecipient<OnRefreshWorkingCopy>
{

    class DifferencesCat
    {
        public required CatResult Original { get; set; }
        public required CatResult Modified { get; set; }
    }

    private readonly LimitedDictionary<string, DifferencesCat?> _differences = new()
    {
        Limit = 20
    };

    public const int ListViewIndex = 0;
    public const int TreeViewIndex = 1;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsTreeView))]
    [NotifyPropertyChangedFor(nameof(IsListView))]
    public partial int SelectedViewIndex { get; set; } = ListViewIndex;

    public bool IsTreeView => SelectedViewIndex == TreeViewIndex;

    public bool IsListView => SelectedViewIndex == ListViewIndex;

    public LocalListViewModel LocalListViewModel { get; set; } = new();

    public LocalTreeViewModel LocalTreeViewModel { get; set; } = new();


    [ObservableProperty]
    public partial string CommitMessage { get; set; } = string.Empty;


    [ObservableProperty]
    public partial string WorkingCopyPath { get; set; } = string.Empty;
    

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

    protected override async Task OnLoaded()
    {
        await base.OnLoaded();
        await CheckWorkingCopyVersion();
        await CheckWorkingCopyRoot();
        await Status();
    }

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


    private async Task CheckWorkingCopyVersion()
    {
        using var context = Engine.Engine.Instance.SimpleContext(Manager.Default.Send(new OnGetDialogHostId(), Token));
        using var wc = context.WorkingCopyContext();
        
        var current = await wc.WcVersion(WorkingCopyPath);
        
        var recommended = await context.DefaultWcVersion();

        Upgradable = recommended.Compare(current) > 0;
    }
    

    private async Task CheckWorkingCopyRoot()
    {
        try
        {
            using var context = Engine.Engine.Instance.SimpleContext(Manager.Default.Send(new OnGetDialogHostId(), Token).Response);
            using var wc = context.WorkingCopyContext();
            var result = await wc.CheckRoot(WorkingCopyPath);
            IsWcRoot = result.IsWcRoot;
        }
        catch (System.Exception e)
        {
            IsWcRoot = false;
        }
        
    }


    [RelayCommand]
    private async Task Revert()
    {
        Dictionary<string, StatusEntry>? items;
        if (IsTreeView)
        {
            items = LocalTreeViewModel.CheckedItems;
        } else if (IsListView)
        {
            items = LocalListViewModel.CheckedItems;
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
            if (item.Value.NodeStatus == NodeStatus.Unversioned)
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
        
        var message = unversioned.Count > 0 ?
            $"The following files will be reverted:\n{string.Join(Environment.NewLine, versioned.Select(e => e.TrimStartString(WorkingCopyPath).TrimStartPathSeparatorChar()))}\n\nThe following files will be skipped:\n{string.Join(Environment.NewLine, unversioned.Select(e => e.TrimStartString(WorkingCopyPath).TrimStartPathSeparatorChar()))}" 
            : $"The following files will be reverted:\n{string.Join(Environment.NewLine, versioned.Select(e => e.TrimStartString(WorkingCopyPath).TrimStartPathSeparatorChar()))}";
        
        
        
        
        var hostId =  Manager.Default.Send(new OnGetDialogHostId(), Token).Response;
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
        

        using var context = Engine.Engine.Instance.SimpleContext(hostId);

        try
        {
            await context.Revert(revertOptions);
            await Status(context);
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

    
    private async Task Status(AsyncContext context, bool dispose = false)
    {
        try
        {

            var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

            var statusOptions = new StatusOptions(
                Path: WorkingCopyPath,
                Revision: new Revision.Working(),
                Depth: Depth.Infinity,
                GetAll: false,
                CheckOutOfDate: false,
                CheckWorkingCopy: false,
                NoIgnore: false,
                IgnoreExternals: true,
                DepthAsSticky: false, Changelist: []);


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
                        Path: WorkingCopyPath,
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
                    Manager.Default.Send(new OnRemoveTab(Token), Manager.MainWindowToken);
                    return;
                }
            }

            var incompleteList = result.Entries.Where(e => e.NodeStatus == NodeStatus.Incomplete).ToList();
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
                    Manager.Default.Send(new OnRemoveTab(Token), Manager.MainWindowToken);
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
                Manager.Default.Send(new OnNotWorkingCopy(WorkingCopyPath), Token);
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
        finally
        {
            if (dispose)
            {
                context.Dispose();
            }
        }
    }

    [RelayCommand]
    public async Task Status()
    {
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

        using var context = Engine.Engine.Instance.SimpleContext(hostId);
        
        await Status(context);
    }

    [RelayCommand]
    private async Task Update()
    {
        try
        {
            using var context = Engine.Engine.Instance.SimpleContext(Manager.Default.Send(new OnGetDialogHostId(), Token).Response);
            
            var opts = new UpdateOptions(
                Paths: [WorkingCopyPath], 
                Revision: new Revision.Head(), 
                Depth: Depth.Infinity, 
                DepthIsSticky: false, 
                IgnoreExternals: false, 
                AllowUnverObstructions: true, 
                AddsAdModification: false, 
                MakeParents: true);
            
            await context.Update(opts);
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
        LocalListViewModel.Update(statusEntries);
        
        LocalTreeViewModel.Update(statusEntries);
    }



    [RelayCommand]
    private async Task Commit()
    {
        try
        {
            if (string.IsNullOrEmpty(CommitMessage))
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = "Empty commit message is not allowed",
                    Type = NotificationType.Warning
                }, Manager.MainWindowToken);
                return;
            }
            var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

            var context = Engine.Engine.Instance.SimpleContext(hostId);

            List<string> unversioned = [];
        
            List<string> missing = [];
        
            List<string> others = [];


            var items = IsTreeView ? LocalTreeViewModel.CheckedItems : LocalListViewModel.CheckedItems;

            foreach (var item in items)
            {
                switch (item.Value.NodeStatus)
                {
                    case NodeStatus.Unversioned:
                        unversioned.Add(item.Key);
                        break;
                    case NodeStatus.Missing:
                        missing.Add(item.Key);
                        break;
                    case NodeStatus.None:
                    case NodeStatus.Normal:
                    case NodeStatus.Added:
                    case NodeStatus.Deleted:
                    case NodeStatus.Replaced:
                    case NodeStatus.Modified:
                    case NodeStatus.Merged:
                    case NodeStatus.Conflicted:
                    case NodeStatus.Ignored:
                    case NodeStatus.Obstructed:
                    case NodeStatus.External:
                    case NodeStatus.Incomplete:
                    default:
                        others.Add(item.Key);
                        break;
                }
            }

            if (missing.Count > 0 && CommitMissingAsDelete)
            {
                await context.Delete(new DeleteOptions(missing.ToArray(), false, false, []));
            
                others.AddRange(missing);
            }

            if (unversioned.Count > 0 && AddUnversionedBeforeCommit)
            {
                foreach (var item in unversioned)
                {
                    await context.Add(new AddOptions(item, Depth.Empty, false, false, false, false));
                }
            
                others.AddRange(unversioned);
            }

            if (others.Count == 0)
            {
                return;
            }


            var commitOptions = new CommitOptions(
                others.ToArray(),
                Depth: Depth,
                KeepLocks: true,
                KeepChangelist: false, 
                CommitAsOperations: true, 
                IncludeFileExternals: true, 
                IncludeDirExternals: true,
                Changelists: [], 
                RevisionPropertyTable: new Dictionary<string, string>(), 
                CommitMessage: CommitMessage);
        
            await context.Commit(commitOptions);
            
            await Status(context);

            Manager.Default.Send(new OnShowToast()
            {
                Content = "Commit successfully",
                Type =  NotificationType.Success,
            }, Manager.MainWindowToken);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to commit: {e.HumanReadableMessage}",
                Type =  NotificationType.Error,
            }, Manager.MainWindowToken);
        }

    }

    public void Receive(OnSelectedItemChanged message)
    {
        if (message.Value is not StatusEntry statusEntry) return;
        SelectedItem = statusEntry.Path;
        
        TryCompareStatusEntry(statusEntry);
    }


    [RelayCommand]
    private void TryCompareStatusEntry(StatusEntry statusEntry)
    {
        var contains = _differences.TryGetValue(statusEntry.Path, out var difference);
        if (contains)
        {
            if (difference is null) return;
        }
        
        Log.Info($"Selected Entry: {statusEntry}");
            
        _differences[statusEntry.Path] = null;
        EditorState = LoadingOrErrorState.MakeLoading();

        Dispatcher.UIThread.InvokeAsync(async () =>
        {

            var success = false;
            AsyncContext? context = null;
            try
            {
                context = Engine.Engine.Instance.SimpleContext(Manager.Default.Send(new OnGetDialogHostId(), Token).Response);
                
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

                var handle = context;
                if (statusEntry.NodeStatus is NodeStatus.Missing or NodeStatus.Deleted)
                {
                    catModified = () => Task.FromResult(new CatResult([], []));
                    catOriginal = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Base(),
                            Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
                        return handle.Cat(catOptions);
                    };
                }
                else if (statusEntry.NodeStatus is NodeStatus.Unversioned or NodeStatus.Added)
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
                        return handle.Cat(catOptions);
                    };
                    catOriginal = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                            Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
                        return handle.Cat(catOptions);
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
                context?.Dispose();
            }
        });
        
    }

    public void Receive(OnRefreshWorkingCopy message)
    {
        Dispatcher.UIThread.InvokeAsync(async () => await Status());
    }
}