using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using Ursa.Controls;
using OperationDepth = SVNexus.Generated.Depth;
namespace SVNexus.ViewModels;


public partial class RevertDialogModel(ViewModelBase parent): DialogModelBase(parent)
{

    public class TargetItemViewModel : StatusEntryItemViewModel
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

    public List<TargetItemViewModel> TargetItems =>
        TargetEntries.Select(i => new TargetItemViewModel()
        {
            Entry = i,
            RelateTo = RelateTo
        }).ToList();


    public required string RelateTo { get; set; }
    
    public required List<StatusEntry> TargetEntries { get; set; }

    [ObservableProperty] 
    [NotifyPropertyChangedFor(nameof(Recursively))] 
    public partial ValidDepth Depth { get; set; } = ValidDepth.Empty;
    
    
    [ObservableProperty]
    public partial bool MetadataOnly { get; set; }
    
    [ObservableProperty]
    public partial bool AddedKeepLocal { get; set; }


    public bool Recursively
    {
        get => Depth == ValidDepth.Infinity;

        set
        {
            if (value)
            {
                Depth = ValidDepth.Infinity;
            }
        }
    }
    
    protected override async Task OnConfirm()
    {
        var options = new RevertOptions(
            TargetEntries.Select(i => i.Path).ToArray(),
            (OperationDepth)Depth,
            MetadataOnly,
            AddedKeepLocal);


        var context = SendMessage(new OnGetContext()).Response;

        await context.Revert(options);
        
        Ok();
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Revert",
        Buttons = DialogButton.None,
        IsCloseButtonVisible = false
    };
}