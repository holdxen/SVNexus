using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
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

public partial class UnlockDialogModel(ViewModelBase parent): DialogModelBase(parent)// ViewModelBase(parent), IDialogContext
{
    // public bool Accept { get; set; }
    
    // public class TargetItemViewModel: StatusEntryItemViewModel
    // {
    //     
    // }
    //
    // public required string RelateTo { get; set; }

    // public required List<StatusEntry> TargetEntries { get; init; }


    public required List<TargetItemViewModel> Targets { get; set; }
        // TargetEntries.Select(i => new TargetItemViewModel()
        // {
        //     Entry = i,
        //     RelateTo = RelateTo
        // }).ToList();
    
    [ObservableProperty]
    public partial bool BreakLock { get; set; }
    
    // [RelayCommand]
    // public void Close()
    // {
    //     Accept = false;
    //     RequestClose?.Invoke(this, null);
    // }
    //
    // public event EventHandler<object?>? RequestClose;


    // [RelayCommand]
    protected override async Task OnConfirm()
    {
        try
        {
            var context = SendMessage(new OnGetContext()).Response;

            var options = new UnlockOptions(Targets.Select(i => i.Path).ToArray(), BreakLock);
            
            await context.Unlock(options);

            Ok();
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to unlock\n{string.Join("\n", Targets.Select(i => i.Path))}\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }

        
        
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Unlock",
        IsCloseButtonVisible = false,
        Buttons = DialogButton.None
    };
}