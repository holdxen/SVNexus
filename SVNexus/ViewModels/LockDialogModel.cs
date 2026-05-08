using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class LockDialogModel(ViewModelBase parent): DialogModelBase(parent)
{
    
    // public bool Accept { get; set; }

    // public class TargetItemViewModel: StatusEntryItemViewModel
    // {
    //     
    // }


    // public required string RelateTo { get; set; }

    // public required List<StatusEntry> TargetEntries { get; init; }


    // public List<TargetItemViewModel> Targets =>
    //     TargetEntries.Select(i => new TargetItemViewModel()
    //     {
    //         Entry = i,
    //         RelateTo = RelateTo
    //     }).ToList();
    
    public required List<TargetItemViewModel> Targets { get; set; }


    [ObservableProperty]
    public partial bool StealLock { get; set; }


    [ObservableProperty] public partial string Comment { get; set; } = string.Empty;
    
    
    [ObservableProperty]
    public partial bool HasComment { get; set; }

    protected override async Task OnConfirm()
    {

        try
        {

            var context = SendMessage(new OnGetContext()).Response;
        
        
            var lockOptions = new LockOptions(Targets.Select(i => i.Path).ToArray(), HasComment ? Comment : null, StealLock);

        
            await context.Lock(lockOptions);
        
            Accept = true;
            Ok();
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to lock\n{string.Join("\n", Targets.Select(i => i.Path))}\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Lock",
        IsCloseButtonVisible = false,
        Buttons = DialogButton.None
    };
}