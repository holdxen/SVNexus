using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Controls.Notifications;
using Avalonia.Controls.Primitives;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using ExCSS;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.Views;
using Ursa.Controls;
using Exception = System.Exception;

namespace SVNexus.ViewModels.WorkingCopy.History;

public partial class HistoryViewModel(ViewModelBase parent): ViewModelMore(parent)
{

    public const int DetailViewIndex = 0;
    public const int ChangesViewIndex = 1;
    public const int SnapshotViewIndex = 2;
    
    public partial class CommitItemViewModel(ViewModelBase parent): ViewModelBase(parent)
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
        
        
        public string? DateTimeText => Entry.Date?.Map(d => DateTimeOffset.FromUnixTimeMilliseconds(d / 1000).UtcDateTime.ToLocalTime().ToString("yyyy-MM-dd HH:mm:ss"));
        
        public ObservableCollection<CommitItemViewModel> Children { get; } = [];

        public ObservableCollection<MenuItemViewModel> MenuItems =>
        [
            new()
            {
                Header = "Copy revision",
                Command = CopyRevisionCommand
            },
            new()
            {
                Header = "Copy message",
                Command = CopyMessageCommand
            },
            new()
            {
                Header = "Copy author",
                Command = CopyAuthorCommand
            },
            new()
            {
                Header = "Update to this",
                Command = UpdateToThisCommand
            }
        ];

        [RelayCommand]
        private async Task UpdateToThis()
        {
            var hostId = SendMessage(new OnGetDialogHostId());
            var path = SendMessage(new OnGetWorkingCopyPath());
            
            var model = new UpdateDialogModel(this)
            {
                TargetItems= 
                [
                    new TargetItemViewModel()
                    {
                        FileName = path,
                        Path = path,
                        TextToolTip = path,
                        KindIcon = NodeKind.Directory.Icon(),
                    }
                ],
                IsNumber = true,
                Number = Revision ?? 1,
            };
            
            await OverlayDialog.ShowModal<UpdateDialog, UpdateDialogModel>(model, hostId, model.OverlayDialogOptions);
        }

        [RelayCommand]
        private async Task CopyRevision()
        {
            await Manager.Default.Send(new ClipBoardMessages.SetText()
            {
                Text = Revision?.ToString() ?? string.Empty
            }, Manager.MainWindowToken);
        }

        [RelayCommand]
        private async Task CopyMessage()
        {
            await Manager.Default.Send(new ClipBoardMessages.SetText()
            {
                Text = Entry.Message ?? string.Empty,
            }, Manager.MainWindowToken);
        }

        [RelayCommand]
        private async Task CopyAuthor()
        {
            await Manager.Default.Send(new ClipBoardMessages.SetText()
            {
                Text = Entry.Author ?? string.Empty,
            }, Manager.MainWindowToken);
        }
    }


    [ObservableProperty]
    public partial int SelectedViewIndex { get; set; } = DetailViewIndex;


    [ObservableProperty] public partial int SelectedCommitItemIndex { get; set; } = -1;

    [ObservableProperty]
    public partial ObservableCollection<CommitItemViewModel> CommitItems { get; set; } = [];
    
    // public string? Url { get; set; }

    // [ObservableProperty] public partial string WorkingCopyPath { get; set; } = string.Empty;
    
    // public required WeakReferenceMessenger Messenger { get; set; }

    [ObservableProperty]
    public partial bool IsLoading { get; set; }

    private uint? _startRevision;
    
    [ObservableProperty]
    public partial HistoryDetailViewModel? DetailViewModel { get; set; }
    
    
    [ObservableProperty]
    public partial HistorySnapshotViewModel? SnapshotViewModel { get; set; }
    
    [ObservableProperty]
    public partial HistoryChangesViewModel? ChangesViewModel { get; set; }
    
    public InfoEntry? ThisEntry { get; set; }

    // private readonly Services.ITabService _tabService;
    //
    // private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    //
    // private readonly IServiceProvider _serviceProvider;
    // private readonly Services.TypeService _typeService;
    //
    // private IServiceScope? _previousScope;

    // public HistoryViewModel(IServiceProvider serviceProvider)
    // {
    //     _serviceProvider = serviceProvider;
    //     _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
    //     _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
    //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
    //     
    //     
    //     Manager.Default.RegisterAllMessages(this, this.GetToken(_typeService));
    // }

    private void SetHistoryChangesViewModel(HistoryChangesViewModel vm)
    {
        var leftPartWidth = ChangesViewModel?.LeftPartRealWidth;
        ChangesViewModel = vm;
        ChangesViewModel.Update();
        if (leftPartWidth != null)
        {
            // ChangesViewModel.LeftPartWidth = new GridLength(leftPartWidth.Value.Value,  GridUnitType.Pixel);
            ChangesViewModel.LeftPartWidth = leftPartWidth.Value;
        }
    }

    partial void OnSelectedCommitItemIndexChanged(int value)
    {
        if (CommitItems.Count <= value || value < 0 || ThisEntry is null) return;
        
        var relateToRoot = ThisEntry.Url.TrimStartString(ThisEntry.RepositoryRootUrl);
        var commitItem = CommitItems[value];
        DetailViewModel = new HistoryDetailViewModel
        {
            Entry = commitItem.Entry,
            RelateToRoot = relateToRoot
        };
        Revision revision = commitItem.Revision is null
            ? new Revision.Head()
            : new Revision.Number(commitItem.Revision.GetValueOrDefault());
        
        var url = ThisEntry.Url;

        SnapshotViewModel = new HistorySnapshotViewModel(this)
        {
            CurrentUrl = url,
            Revision = revision,
        };

        if (commitItem.Revision is null)
        {
            ChangesViewModel = null;
            return;
        }
        
        // 对比视图，需要找到对比的revision
        if (value == CommitItems.Count - 1)
        {
            // 最后一个，需要加载更多的历史

            if (commitItem.Revision == 1)
            {
                SetHistoryChangesViewModel(new HistoryChangesViewModel(this)
                {
                    CurrentRevision = commitItem.Revision.GetValueOrDefault(),
                    CompareRevision = null,
                    RelateToRoot = relateToRoot,
                    LogChangedPathEntries = commitItem.Entry.ChangedPathEntries,
                    RootUrl = ThisEntry.RepositoryRootUrl,
                });
            }
            else
            {
                ChangesViewModel = null;
                Dispatcher.UIThread.InvokeAsync(async () =>
                {
                    while (SelectedCommitItemIndex == value && CommitItems.LastOrDefault()?.Revision != 1)
                    {
                        await Log(10);
                        if (CommitItems.Count <= value + 1 || SelectedCommitItemIndex != value) continue;
                        SetHistoryChangesViewModel(new HistoryChangesViewModel(this)
                        {
                            CurrentRevision = commitItem.Revision.GetValueOrDefault(),
                            CompareRevision = CommitItems[value + 1].Revision,
                            RelateToRoot = relateToRoot,
                            LogChangedPathEntries = commitItem.Entry.ChangedPathEntries,
                            RootUrl = ThisEntry.RepositoryRootUrl,
                        });
                        break;
                    }
                });
            }
        }
        else
        {
            SetHistoryChangesViewModel(new HistoryChangesViewModel(this)
            {
                CurrentRevision = commitItem.Revision.GetValueOrDefault(),
                CompareRevision = CommitItems[value + 1].Revision.GetValueOrDefault(),
                RelateToRoot = relateToRoot,
                LogChangedPathEntries = commitItem.Entry.ChangedPathEntries,
                RootUrl = ThisEntry.RepositoryRootUrl,
            });
        }



        // if (commitItem.Revision is not null)
        // {
        //     if (value == CommitItems.Count - 1)
        //     {
        //         Dispatcher.UIThread.InvokeAsync(async () =>
        //         {
        //             while (SelectedCommitItemIndex == value)
        //             {
        //                 await Log(10);
        //                 if (CommitItems.Count - 1 > value)
        //                 {
        //                     break;
        //                 }
        //             }
        //
        //             if (SelectedCommitItemIndex != value)
        //             {
        //                 return;
        //             }
        //             var item = CommitItems[value + 1];
        //             if (item.Revision is null)
        //             {
        //                 return;
        //             }
        //             
        //             ChangesViewModel = new HistoryChangesViewModel
        //             {
        //                 CurrentRevision = new Revision.Number(commitItem.Revision.GetValueOrDefault()),
        //                 CompareRevision = new Revision.Number(item.Revision.GetValueOrDefault()),
        //                 RelateToRoot = relateToRoot,
        //                 LogChangedPathEntries = commitItem.Entry.ChangedPathEntries,
        //                 CurrentPath = string.Empty,
        //             };
        //             ChangesViewModel.Update();
        //             
        //         });
        //     }
        //     else
        //     {
        //         var item = CommitItems[value + 1];
        //         if (item.Revision is null)
        //         {
        //             return;
        //         }
        //             
        //         ChangesViewModel = new HistoryChangesViewModel
        //         {
        //             CurrentRevision = new Revision.Number(commitItem.Revision.GetValueOrDefault()),
        //             CompareRevision = new Revision.Number(item.Revision.GetValueOrDefault()),
        //             RelateToRoot = relateToRoot,
        //             LogChangedPathEntries = commitItem.Entry.ChangedPathEntries,
        //             CurrentPath = string.Empty,
        //         };
        //         ChangesViewModel.Update();
        //     }
        // }
        // else
        // {
        //     ChangesViewModel = null;
        // }
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
        LogCommand.ExecuteOrNothing(20);
    }


    private void HandleException(Exception e)
    {
        var handled = e.Handle(svnExceptionHandler: error =>
        {
            // var path = SendMessage(new OnGetWorkingCopyPath());
            if (!ExceptionExtension.SvnErrnoConstants.IsWcNotWorkingCopy(error.Code)) return false;
            SendMessage(new OnNotWorkingCopy());
            // Manager.Default.Send(new OnNotWorkingCopy(path), _typeService.Get(this));
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
    
    


    private async Task CheckUrl()
    {
        if (ThisEntry is not null)
        {
            return;
        }
        var hostId = SendMessage(new OnGetDialogHostId());
        var path = SendMessage(new OnGetWorkingCopyPath());

        using var context = EngineBackend.Instance.SimpleContext(hostId);

        try
        {
            var opts = new InfoOptions(
                Path: path,
                PegRevision: new Revision.Unspecified(),
                Revision: new Revision.Unspecified(),
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
        }
        catch (Exception e)
        {
            HandleException(e);
        }

        
        
    }

    private readonly Stack<CommitItemViewModel> _parentStack = [];

    private void HandleLogEntries(LogEntry[] logs, IList<CommitItemViewModel> commitItems, Stack<CommitItemViewModel> parentStack)
    {
        foreach (var entry in logs)
        {
            if (entry.Revision is null)
            {
                parentStack.Pop();
            } 
            else if (entry.HasChildren)
            {
                var item = new CommitItemViewModel(this)
                {
                    Entry = entry
                };
                var last = parentStack.LastOrDefault();
                if (last is not null)
                {
                    last.Children.Add(item);
                }
                else
                {
                    commitItems.Add(item);
                }
                parentStack.Push(item);

                foreach (var change in item.Entry.ChangedPathEntries)
                {
                }
                // _parent = item;
                // CommitItems.Add(item);
            }
            else if (parentStack.Count > 0)
            {
                parentStack.Last().Children.Add(new CommitItemViewModel(this)
                {
                    Entry = entry,
                });
            }
            else
            {
                commitItems.Add(new CommitItemViewModel(this)
                {
                    Entry = entry
                });
            }
        }
    }

    private Tuple<uint, uint>? _logsRange; 


    [RelayCommand]
    private async Task LogCache()
    {
        await CheckUrl();
        if (ThisEntry is null)
        {
            return;
        }

        if (_logsRange?.Item2 <= 2)
        {
            return;
        }
        
        
        var hostId = SendMessage(new OnGetDialogHostId());
        using var context = EngineBackend.Instance.SimpleContext(hostId);

        Logger.Info($"Range: {_logsRange}");

        var logs = await context.LogCacheFillLocal(
            SeaDatabaseConnection.Default, 
            ThisEntry.RepositoryUuid,
            SendMessage(new OnGetWorkingCopyPath()),
            _logsRange?.Item2 - 1, 
            64);
        

        if (logs.Length == 0)
        {
            return;
        }
        
        _logsRange = new Tuple<uint, uint>(_logsRange?.Item1 ?? logs.First().Entry.Revision.GetValueOrDefault(), logs.Last().Entry.Revision.GetValueOrDefault());
        
        CommitItems.AddRange(logs.Select(i => new CommitItemViewModel(this)
        {
            Entry = i.Entry
        }));
    }


    [RelayCommand]
    private async Task Log(uint limit)
    {
        // await LogDirectly(limit);
        try
        {
            await LogCache();
        }
        catch (Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = "Failed to log repository: " + e.HumanReadableMessage,
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }

    private async Task LogDirectly(uint limit)
    {

        AsyncContext? context = null;
        try
        {
            if (_startRevision <= 1)
            {
                return;
            }

            await CheckUrl();
            
            if (ThisEntry is null)
            {
                return;
            }

            // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
            var hostId = SendMessage(new OnGetDialogHostId());

            context = EngineBackend.Instance.SimpleContext(hostId);


            var logOptions = new LogOptions(
                Targets: [ThisEntry.Url],
                PegRevision: new Revision.Head(),
                Limit: limit,
                Revisions:
                [
                    new RevisionRange(
                        Start: _startRevision is null
                            ? new Revision.Head()
                            : new Revision.Number(_startRevision.GetValueOrDefault() - 1), End: new Revision.Number(0))
                ],
                DiscoverChangedPaths: true,
                StrictNodeHistory: false,
                IncludeMergedRevisions: true,
                RevisionsProperties: ["svn:author", "svn:log", "svn:date"]);


            await context.LogNext(logOptions, new LogReceiverDelegate()
            {
                OnLogEntryAction = entry =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        if (entry.Revision is null)
                        {
                            _parentStack.Pop();
                        } 
                        else if (entry.HasChildren)
                        {
                            var item = new CommitItemViewModel(this)
                            {
                                Entry = entry
                            };
                            var last = _parentStack.LastOrDefault();
                            if (last is not null)
                            {
                                last.Children.Add(item);
                            }
                            else
                            {
                                CommitItems.Add(item);
                            }
                            _parentStack.Push(item);
                            // _parent = item;
                            // CommitItems.Add(item);
                        }
                        else if (_parentStack.Count > 0)
                        {
                            _parentStack.Last().Children.Add(new CommitItemViewModel(this)
                            {
                                Entry = entry,
                            });
                        }
                        else
                        {
                            CommitItems.Add(new CommitItemViewModel(this)
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
        }
        catch (Exception e)
        {
            HandleException(e);
        }
        finally
        {
            context?.Dispose();
        }



    }

    protected override async Task LoadOnce()
    {
        await LogCommand.ExecuteAsync(20);
    }

}