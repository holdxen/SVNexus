using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Controls.Primitives;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using ExCSS;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using Exception = System.Exception;
using Notification = Ursa.Controls.Notification;

namespace SVNexus.ViewModels.WorkingCopy.Remote;

public partial class RemoteViewModel: ViewModelBase
{
    
    public const int DetailViewIndex = 0;
    public const int ChangesViewIndex = 1;
    public const int SnapshotViewIndex = 2;
    
    public partial class CommitItemViewModel: ViewModelLite
    {
        
        [ObservableProperty]
        public required partial LogEntry Entry { get; set; }
        
        public uint? Revision => Entry.Revision;
        
        public string Author => Entry.Author ?? string.Empty;
        
        public string DisplayMessage
        {
            get
            {
                if (string.IsNullOrWhiteSpace(Entry.Message))
                {
                    return string.Empty;
                }

                var index = Entry.Message.IndexOf("\n\n", StringComparison.Ordinal);
                return index >= 0 ? Entry.Message[..index].Replace("\n", string.Empty) : Entry.Message.Replace("\n", string.Empty);
            }
        }
        
        
        public string DateTimeText => DateTimeOffset.FromUnixTimeMilliseconds(Entry.Date.GetValueOrDefault() / 1000).UtcDateTime.ToString("u");
        
        public ObservableCollection<CommitItemViewModel> Children { get; } = [];

    }


    [ObservableProperty]
    public partial int SelectedViewIndex { get; set; } = DetailViewIndex;


    [ObservableProperty] public partial int SelectedCommitItemIndex { get; set; } = -1;

    public ObservableCollection<CommitItemViewModel> CommitItems { get; } = [];
    
    // public string? Url { get; set; }

    [ObservableProperty] public partial string WorkingCopyPath { get; set; } = string.Empty;
    
    // public required WeakReferenceMessenger Messenger { get; set; }

    [ObservableProperty]
    public partial bool IsLoading { get; set; }

    private uint? _startRevision;
    
    [ObservableProperty]
    public partial RemoteDetailViewModel? DetailViewModel { get; set; }
    
    
    [ObservableProperty]
    public partial RemoteSnapshotViewModel? SnapshotViewModel { get; set; }
    
    [ObservableProperty]
    public partial RemoteChangesViewModel? ChangesViewModel { get; set; }
    
    public InfoEntry? ThisEntry { get; set; }

    partial void OnSelectedCommitItemIndexChanged(int value)
    {
        if (CommitItems.Count <= value || value < 0 || ThisEntry is null) return;
        var relateToRoot = ThisEntry.Url.TrimStartString(ThisEntry.ReposRootUrl);
        var commitItem = CommitItems[value];
        DetailViewModel = new RemoteDetailViewModel
        {
            Entry = commitItem.Entry,
            RelateToRoot = relateToRoot
        };
        SnapshotViewModel = new RemoteSnapshotViewModel()
        {
            // Messenger = Messenger,
            Revision = commitItem.Revision is null ? new Revision.Head() : new Revision.Number(commitItem.Revision.GetValueOrDefault()),
            Url = ThisEntry.Url
        };
        if (commitItem.Revision is not null)
        {
            if (value == CommitItems.Count - 1)
            {
                Dispatcher.UIThread.InvokeAsync(async () =>
                {
                    while (SelectedCommitItemIndex == value)
                    {
                        await Log(10);
                        if (CommitItems.Count - 1 > value)
                        {
                            break;
                        }
                    }

                    if (SelectedCommitItemIndex != value)
                    {
                        return;
                    }
                    var item = CommitItems[value + 1];
                    if (item.Revision is null)
                    {
                        return;
                    }
                    
                    ChangesViewModel = new RemoteChangesViewModel
                    {
                        CurrentRevision = new Revision.Number(commitItem.Revision.GetValueOrDefault()),
                        CompareRevision = new Revision.Number(item.Revision.GetValueOrDefault()),
                        RelateToRoot = relateToRoot,
                        LogChangedPathEntries = commitItem.Entry.ChangedPathEntries
                    };
                    ChangesViewModel.Update();
                    
                });
            }
            else
            {
                var item = CommitItems[value + 1];
                if (item.Revision is null)
                {
                    return;
                }
                    
                ChangesViewModel = new RemoteChangesViewModel
                {
                    CurrentRevision = new Revision.Number(commitItem.Revision.GetValueOrDefault()),
                    CompareRevision = new Revision.Number(item.Revision.GetValueOrDefault()),
                    RelateToRoot = relateToRoot,
                    LogChangedPathEntries = commitItem.Entry.ChangedPathEntries
                };
                ChangesViewModel.Update();
            }
        }
        else
        {
            ChangesViewModel = null;
        }
    }

    // public void UpdateViewModel(int commitItemIndex, int viewIndex)
    // {
    //     if (CommitItems.Count <= commitItemIndex || commitItemIndex < 0 || ThisEntry is null) return;
    //     if (viewIndex == DetailViewIndex)
    //     {
    //         var r = ThisEntry.Url.TrimStartString(ThisEntry.ReposRootUrl);
    //         DetailViewModel = new RemoteDetailViewModel
    //         {
    //             Entry = CommitItems[viewIndex].Entry,
    //             RelativeToRoot = r
    //         };
    //     }
    //     else if (viewIndex == SnapshotViewIndex)
    //     {
    //         SnapshotViewModel = new RemoteSnapshotViewModel();
    //     }
    // }


    public void OnDataGridVerticalScrollValueChanged(double value, double maximum)
    {
        if (!((maximum - value) / maximum < 0.1)) return;
        if (!IsLoading)
        {
            Dispatcher.UIThread.InvokeAsync(async () => await Log(20));
        }
    }


    private void HandleException(Exception e)
    {
        var handled = e.Handle(svnExceptionHandler: error =>
        {
            if (!ExceptionExtension.SvnErrnoConstants.IsWcNotWorkingCopy(error.Code)) return false;
            Manager.Default.Send(new OnNotWorkingCopy(WorkingCopyPath), Token);
            return true;
        });
        if (!handled)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to query:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error,
            }, Manager.MainWindowToken);
        }
    }
    
    


    private async Task<bool> CheckUrl()
    {
        if (ThisEntry is not null)
        {
            return true;
        }
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

        using var context = Engine.Engine.Instance.SimpleContext(hostId);

        try
        {
            var opts = new InfoOptions(
                Path: WorkingCopyPath,
                PegRevision: new Revision.Working(),
                Revision: new Revision.Head(),
                Depth: Depth.Empty,
                FetchExcluded: false,
                FetchActualOnly: true,
                IncludeExternals: false,
                Changelists: []
            );
            var result = await context.Info(opts);
            if (result.Entries.Count != 1)
            {
                throw new Exception("Failed to query information");
            }

            ThisEntry = result.Entries.First().Value;

            return true;
        }
        catch (Exception e)
        {
            HandleException(e);
            return false;
        }

        
        
    }

    private CommitItemViewModel? _parent;


    private async Task Log(uint limit)
    {

        AsyncContext? context = null;
        try
        {
            IsLoading = true;
            if (_startRevision is 1)
            {
                return;
            }
            if (!await CheckUrl())
            {
                return;
            }

            var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

            context = Engine.Engine.Instance.SimpleContext(hostId);


            var logOptions = new LogOptions(
                Targets: [ThisEntry!.Url],
                PegRevision: new Revision.Head(),
                Limit: limit,
                Revsions:
                [
                    new RevisionRange(
                        Start: _startRevision is null
                            ? new Revision.Head()
                            : new Revision.Number(_startRevision.GetValueOrDefault() + 1), End: new Revision.Number(1))
                ],
                DiscoverChangedPaths: true,
                StrictNodeHistory: false,
                IncludeMergedRevisions: true,
                RevisionsProperties: ["svn:author", "svn:log", "svn:date"]);


            // var result = await context.Log(logOptions);
            // Console.WriteLine(result.LogEntries.Length);


            await context.LogNext(logOptions, new LogReceiverDelegate()
            {
                OnLogEntryAction = entry =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        if (entry.Revision is null)
                        {
                            _parent = null;
                        } 
                        else if (entry.HasChildren)
                        {
                            var item = new CommitItemViewModel()
                            {
                                Entry = entry
                            };
                            _parent = item;
                            CommitItems.Add(item);
                        }
                        else if (_parent is not null)
                        {
                            _parent.Children.Add(new CommitItemViewModel()
                            {
                                Entry = entry,
                            });
                        }
                        else
                        {
                            CommitItems.Add(new CommitItemViewModel()
                            {
                                Entry = entry
                            });
                        }

                        if (entry.Revision is not null)
                        {
                            _startRevision = entry.Revision;
                        }
                    });
                }
            });

            Console.WriteLine("Log next finished");
        }
        catch (Exception e)
        {
            HandleException(e);
        }
        finally
        {
            IsLoading = false;
            context?.Dispose();
        }



    }

    protected override async Task OnLoaded()
    {
        await base.OnLoaded();
        await Log(20);
    }

}