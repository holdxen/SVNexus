using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Reflection.Metadata.Ecma335;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using HarfBuzzSharp;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class CommitDialogModel(ViewModelBase parent): ViewModelBase(parent), IDialogContext
{
    public partial class ToBeCommittedItem: ViewModelBase
    {
        public required StatusEntry StatusEntry { get; set; }
        
        [ObservableProperty]
        public partial string WorkingCopyPath { get; set; } = string.Empty;


        public string NodeKindIcon => StatusEntry.NodeKind.Icon();
        
        public string NodeStatusIcon => StatusEntry.NodeStatus.Icon();

        public string Name => StatusEntry.Path.GetFileName();
        
        public string RelativePath { get; set; } = string.Empty;
    }
    
    public override Type? ViewType { get; set; } = typeof(CommitDialog);
    
    public static readonly Type DepthType = typeof(Depth);

    public required StatusEntry[] Targets { get; init; }

    [ObservableProperty]
    public partial Depth Depth { get; set; } = Depth.Infinity;
    
    
    private readonly SingleTaskQueue _singleTaskQueue = new();

    public ObservableCollection<ToBeCommittedItem> ToBeCommittedItems { get; } = [];

    // [ObservableProperty] 
    // public partial bool DeleteMissing { get; set; } = true;
    //
    // [ObservableProperty]
    // public partial bool AddUnversioned { get; set; } = true;

    [ObservableProperty]
    public partial bool NoLock { get; set; }
    
    [ObservableProperty]
    public partial bool IncludeExternals { get; set; }
    
    [ObservableProperty]
    public partial bool IsLoading { get; set; }

    [Required]
    public string CommitMessage
    {
        get;
        set => SetProperty(ref field, value);
    } = string.Empty;


    // public bool Ready
    // {
    //     get
    //     {
    //         if (!DeleteMissing && !AddUnversioned) return true;
    //         return !IsLoading;
    //     }
    // }
    //
        
    private Dictionary<string, StatusEntry> _entries = [];

    [ObservableProperty]
    public partial bool IsCommitting { get; set; }

    // private Services.ITabService _tabService;
    // public CommitDialogModel(IServiceProvider serviceProvider)
    // {
    //     _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
    // }
    //
    partial void OnDepthChanged(Depth value)
    {
        _singleTaskQueue.Run(LoadCommitItems);
    }


    // private async Task CommitDirectly(CancellationToken token)
    // {
    //         
    // }
    //
    // private async Task CommitDelay()
    // {
    //     if (IsLoading)
    //     {
    //         return;
    //     }
    //     
    //     var context = Engine.Engine.Instance.SimpleContext(SendMessage(new OnGetDialogHostId()));
    //
    //     var tobeDelete = new List<string>();
    //     foreach (var entry in _entries.Values)
    //     {
    //         if (entry.NodeStatus == NodeStatus.Missing)
    //         {
    //             tobeDelete.Add(entry.Path);
    //         }
    //
    //         var addOptions = new AddOptions(entry.Path, Depth.Infinity, false, false, false, true);
    //         await context.Add(addOptions);
    //     }
    //
    //     await context.Delete(new DeleteOptions(tobeDelete.ToArray(), false, false, []));
    //
    //     var commitOptions = new CommitOptions(Targets.ToArray(), Depth, true, true, true, true, true, [], [], CommitMessage);
    //     
    //     await context.Commit(commitOptions);
    //     
    //     IsCommitting = false;
    // }

    [RelayCommand]
    private async Task Commit()
    {
        // if (IsLoading)
        // {
        //     IsCommitting = true;
        //     _singleTaskQueue.QueueEmpty += CommitDelay;
        // }
        // else
        // {
        //     IsCommitting = true;
        //     await _singleTaskQueue.Run(CommitDirectly);
        // }
        //

        if (!ValidateAllProperty(out _))
        {
            return;
        }
        
        await _singleTaskQueue.Run(async token =>
        {
            try
            {
                IsCommitting = true;

                var hostId = SendMessage(new OnGetDialogHostId());
            
                var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

                var commitOptions = new CommitOptions(Targets.Where(i => i.NodeStatus is not (WorkingCopyStatus.Unversioned or WorkingCopyStatus.Missing)).Select(i => i.Path).ToArray(),
                    Depth,
                    NoLock,
                    false,
                    true,
                    IncludeExternals,
                    IncludeExternals,
                    null,
                    null,
                    CommitMessage
                );


                await context.Commit(commitOptions);
            }
            catch (System.Exception e)
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = $"Failed to commit: {e.HumanReadableMessage}",
                    Type =  NotificationType.Error,
                }, Manager.MainWindowToken);
            }
            finally
            {
                IsCommitting = false;
            }


        });
    }


    private async Task LoadCommitItems(CancellationToken token)
    {
        if (token.IsCancellationRequested)
        {
            return;
        }
        IsLoading = true;
        var depth = Depth;
        var commitPaths = Targets.ToList();
        var includeExternals = IncludeExternals;
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token);
        var hostId = SendMessage(new OnGetDialogHostId());

        try
        {

            var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

            if (token.IsCancellationRequested)
            {
                return;
            }

            var entries = new Dictionary<string, StatusEntry>();
            foreach (var path in commitPaths)
            {
                if (token.IsCancellationRequested)
                {
                    return;
                }

                var statusOptions = new StatusOptions(path.Path, new Revision.Working(), depth, false, false, false, false, includeExternals, false, null);

                var result = await context.Status(statusOptions);

                foreach (var entry in result.Entries)
                {
                    entries[entry.Path] = entry;
                }
            }

            if (token.IsCancellationRequested)
            {
                return;
            }

            ToBeCommittedItems.Clear();

            var containsUnavaliable = false;
            foreach (var entry in entries.Values)
            {
                if (entry.NodeStatus is WorkingCopyStatus.Missing or WorkingCopyStatus.Unversioned)
                {
                    containsUnavaliable = true;
                    continue;
                }
                ToBeCommittedItems.Add(new ToBeCommittedItem()
                {
                    StatusEntry = entry,
                });
            }

            _entries = entries;
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Type = NotificationType.Error,
                Content = $"Failed to query commit items: {e.HumanReadableMessage}"
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
        await _singleTaskQueue.Run(LoadCommitItems);
    }

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, EventArgs.Empty);
    }

    public event EventHandler<object?>? RequestClose;
}