using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;

namespace SVNexus.ViewModels;

public partial class UnlockDialogModel(ViewModelBase parent): ViewModelBase(parent), IDialogContext
{
    public bool Accept { get; set; }
    
    public class TargetItemViewModel: StatusEntryItemViewModel
    {
        
    }

    public required string RelateTo { get; set; }

    public required List<StatusEntry> TargetEntries { get; init; }


    public List<TargetItemViewModel> Targets =>
        TargetEntries.Select(i => new TargetItemViewModel()
        {
            Entry = i,
            RelateTo = RelateTo
        }).ToList();
    
    public bool BreakLock { get; set; }
    
    [RelayCommand]
    public void Close()
    {
        Accept = false;
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;


    [RelayCommand]
    private async Task Confirm()
    {
        var context = SendMessage(new OnGetContext()).Response;

        var options = new UnlockOptions(Targets.Select(i => i.Entry.Path).ToArray(), BreakLock);

        await context.Unlock(options);
        
        Accept = true;
        RequestClose?.Invoke(this, null);
    }
}