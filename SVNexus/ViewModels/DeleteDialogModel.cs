using System;
using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using Ursa.Controls;
using Exception = SVNexus.Generated.Exception;

namespace SVNexus.ViewModels;

public partial class DeleteDialogModel(ViewModelBase parent): DialogModelBase(parent)
{

    // public class TargetItemViewModel : StatusEntryItemViewModel
    // {
    //     
    // }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        IsCloseButtonVisible = false,
        Buttons = DialogButton.None,
        Title = "Delete",
        Mode = DialogMode.Warning
    };

    [ObservableProperty]
    public partial bool KeepLocal { get; set; }
    
    [ObservableProperty]
    public partial bool Force { get; set; }


    // public required List<StatusEntry> TargetEntries { get; set; }
    //
    //
    // public required string RelateTo { get; set; }
    //
    public required List<TargetItemViewModel> Targets { get; set; }
    
    // [ObservableProperty]
    // public required partial string[] Paths { get; set; }
    
    
    [ObservableProperty]
    public partial bool Commited { get; set; }

    [ObservableProperty] public partial string CommitMessage { get; set; } = string.Empty;
    
    
    protected override async Task OnConfirm()
    {
        try
        {
            var deleteOptions = new DeleteOptions(Targets.Select(i => i.Path).ToArray(), Force, KeepLocal, null, CommitMessage);

            var context = SendMessage(new OnGetContext()).Response;
        
            await context.Delete(deleteOptions);
            
            Ok();
        }
        catch (Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Type = NotificationType.Error,
                Content = "Failed to delete: " + e.HumanReadableMessage
            }, Manager.MainWindowToken);
        }
    }
}