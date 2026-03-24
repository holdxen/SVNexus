using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
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
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class CommitDialogModel: ViewModelBase, IDialogContext
{
    public partial class ToBeCommittedItem: ViewModelBase
    {
        public required StatusEntry StatusEntry { get; set; }
        
        [ObservableProperty]
        public partial string WorkingCopyPath { get; set; } = string.Empty;


        public string NodeKindIcon => StatusEntry.NodeKind.NodeKindIcon();
        
        public string NodeStatusIcon => StatusEntry.NodeStatus.NodeStatusIcon();

        public string Name => StatusEntry.Path.GetFileName();
        
        public string RelativePath { get; set; } = string.Empty;
    }
    
    public override Type? ViewType { get; set; } = typeof(CommitDialog);
    
    public static readonly Type DepthType = typeof(Depth);

    public required string[] Targets { get; init; }

    [ObservableProperty]
    public partial Depth Depth { get; set; } = Depth.Infinity;
    
    
    private readonly SingleTaskQueue _singleTaskQueue = new();

    public ObservableCollection<ToBeCommittedItem> ToBeCommittedItems { get; } = [];

    [ObservableProperty] 
    public partial bool DeleteMissing { get; set; } = true;

    [ObservableProperty]
    public partial bool AddUnversioned { get; set; } = true;

    [ObservableProperty]
    public partial bool NoLock { get; set; }
    
    [ObservableProperty]
    public partial bool IncludeExternals { get; set; }
    
    [ObservableProperty]
    public partial bool IsLoading { get; set; }

    [ObservableProperty] public partial string CommitMessage { get; set; } = string.Empty;


    public bool Ready
    {
        get
        {
            if (!DeleteMissing && !AddUnversioned) return true;
            return !IsLoading;
        }
    }
    
        
    private Dictionary<string, StatusEntry> _entries = new();

    [ObservableProperty]
    public partial bool IsCommitting { get; set; }

    partial void OnDepthChanged(Depth value)
    {
        _singleTaskQueue.Run(LoadCommitItems);
    }


    private async Task CommitDirectly(CancellationToken token)
    {
        
    }

    private async Task CommitDelay()
    {
        if (!Ready)
        {
            return;
        }
        
        var context = Engine.Engine.Instance.SimpleContext();

        var tobeDelete = new List<string>();
        foreach (var entry in _entries.Values)
        {
            if (DeleteMissing && entry.NodeStatus == NodeStatus.Missing)
            {
                tobeDelete.Add(entry.Path);
            }

            if (!AddUnversioned || entry.NodeStatus != NodeStatus.Unversioned) continue;
            var addOptions = new AddOptions(entry.Path, Depth, false, false, false, true);
            await context.Add(addOptions);
        }

        await context.Delete(new DeleteOptions(tobeDelete.ToArray(), false, false, []));

        var commitOptions = new CommitOptions(Targets.ToArray(), Depth, true, true, true, true, true, [], [], CommitMessage);
        
        await context.Commit(commitOptions);
        
        IsCommitting = false;
    }

    [RelayCommand]
    private async Task Commit()
    {
        if (!DeleteMissing && !AddUnversioned)
        {
            await _singleTaskQueue.Run(CommitDirectly);
        }
        else if (IsLoading)
        {
            IsCommitting = true;
            _singleTaskQueue.QueueEmpty += CommitDelay;
        }
        else
        {
            IsCommitting = true;
            _singleTaskQueue.QueueEmpty -= CommitDelay;
            await CommitDelay();
        }
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
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token);

        try
        {

            var context = Engine.Engine.Instance.SimpleContext(hostId);

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

                var statusOptions = new StatusOptions(path, new Revision.Working(), depth, false, false, false, false,
                    false, false, []);

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

            foreach (var entry in entries.Values)
            {
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


    protected override async Task OnLoaded()
    {
        await base.OnLoaded();
        await _singleTaskQueue.Run(LoadCommitItems);
    }

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, EventArgs.Empty);
    }

    public event EventHandler<object?>? RequestClose;
}