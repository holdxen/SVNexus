using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.Views;
using Ursa.Controls;
using OperationDepth = SVNexus.Generated.Depth;

namespace SVNexus.ViewModels;

public partial class CommitDialogModel(ViewModelBase parent): DialogModelBase(parent)
{
    public override OverlayDialogOptions OverlayDialogOptions { get; } =  new()
    {
        IsCloseButtonVisible = false,
        Title = "Commit",
        Buttons = DialogButton.None,
        CanDragMove = true,
        CanResize = true,
        StyleClass = "Fixed"
    };

    public class TargetItemViewModel: StatusEntryItemViewModel
    {
    }

    public enum ValidDepth
    {
        Empty = OperationDepth.Empty,
        Files = OperationDepth.Files,
        Immediates = OperationDepth.Immediates,
        Infinity = OperationDepth.Infinity,
    }
    
    public static Type DepthType { get; } = typeof(ValidDepth);


    
    public override Type? ViewType { get; set; } = typeof(CommitDialog);
    
    // public static readonly Type DepthType = typeof(Depth);

    public required StatusEntry[] Targets { get; init; }

    [ObservableProperty]
    public partial ValidDepth Depth { get; set; } = ValidDepth.Infinity;
    
    
    private readonly SingleTaskQueue _singleTaskQueue = new();

    public List<TargetItemViewModel> DisplayTargetItems =>
        ShowActualTargetItem ? TargetItems : Targets.Select(i => new TargetItemViewModel()
        {
            Entry = i,
            RelateTo = RelateTo
        }).ToList();

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(DisplayTargetItems))]
    public partial List<TargetItemViewModel> TargetItems { get; set; } = [];
    
    public required string RelateTo { get; init; } = string.Empty;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(DisplayTargetItems))]
    // [NotifyPropertyChangedFor(nameof(TargetItemText))]
    public partial bool ShowActualTargetItem { get; set; } = true;
    
    // public string TargetItemText => ShowActualTargetItem ? "Actual items:" : "Selected items:";

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

    partial void OnDepthChanged(ValidDepth value)
    {
        _singleTaskQueue.Run(LoadCommitItems);
    }


    [RelayCommand]
    private async Task Commit()
    {
        if (!ValidateAllProperty(out _))
        {
            return;
        }
        
        await _singleTaskQueue.RunAndWait(async token =>
        {
            try
            {
                var hostId = SendMessage(new OnGetDialogHostId());
            
                var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

                var commitOptions = new CommitOptions(Targets.Where(i => i.NodeStatus is not (WorkingCopyStatus.Unversioned or WorkingCopyStatus.Missing)).Select(i => i.Path).ToArray(),
                    (OperationDepth)Depth,
                    NoLock,
                    false,
                    true,
                    IncludeExternals,
                    IncludeExternals,
                    null,
                    CommitMessage
                );


                await context.Commit(commitOptions);
                Ok();
            }
            catch (System.Exception e)
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = $"Failed to commit: {e.HumanReadableMessage}",
                    Type =  NotificationType.Error,
                }, Manager.MainWindowToken);
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

                var statusOptions = new StatusOptions(path.Path, new Revision.Working(), (OperationDepth)depth, false, false, false, false, includeExternals, false, null);

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


            TargetItems = entries.Values.Where(entry => entry.NodeStatus is not (
                WorkingCopyStatus.Missing or 
                WorkingCopyStatus.Unversioned or 
                WorkingCopyStatus.Normal or 
                WorkingCopyStatus.None)).Select(i => new TargetItemViewModel()
            {
                Entry = i,
                RelateTo = RelateTo
            }).ToList();

            // TargetItems.Clear();

            // foreach (var entry in entries.Values.Where(entry => entry.NodeStatus is not (WorkingCopyStatus.Missing or WorkingCopyStatus.Unversioned or WorkingCopyStatus.Normal or WorkingCopyStatus.None)))
            // {
            //     TargetItems.Add(new TargetItemViewModel()
            //     {
            //         Entry = entry,
            //         RelateTo = RelateTo
            //     });
            // }

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

    protected override Task OnConfirm()
    {
        return Commit();
    }
}