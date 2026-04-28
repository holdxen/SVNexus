using System.Collections.Generic;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;

namespace SVNexus.ViewModels;

public partial class DeleteDialogModel(ViewModelBase parent): DialogModelBase(parent)
{

    public class TargetItemViewModel : StatusEntryItemViewModel
    {
        
    }
    
    [ObservableProperty]
    public partial bool KeepLocal { get; set; }
    
    [ObservableProperty]
    public partial bool Force { get; set; }


    public required List<StatusEntry> TargetEntries { get; set; }
    
    
    public required string RelateTo { get; set; }

    public List<TargetItemViewModel> TargetItems => TargetEntries.Select(i => new TargetItemViewModel()
    {
        Entry = i,
        RelateTo = RelateTo
    }).ToList();
    
    // [ObservableProperty]
    // public required partial string[] Paths { get; set; }
    
    
    [ObservableProperty]
    public partial bool Commited { get; set; }

    [ObservableProperty] public partial string CommitMessage { get; set; } = string.Empty;
    
    
    protected override async Task OnConfirm()
    {
        var deleteOptions = new DeleteOptions(TargetEntries.Select(i => i.Path).ToArray(), Force, KeepLocal, null, CommitMessage);

        var context = SendMessage(new OnGetContext()).Response;
        
        await context.Delete(deleteOptions);
    }
}