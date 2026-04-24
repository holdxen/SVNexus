using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using Ursa.Controls;
using OperationDepth = SVNexus.Generated.Depth;

namespace SVNexus.ViewModels;

public partial class UpdateDialogModel(ViewModelBase parent): DialogModelBase(parent)
{
    public enum ValidDepth
    {
        Empty = OperationDepth.Empty,
        Files = OperationDepth.Files,
        Immediates = OperationDepth.Immediates,
        Infinity = OperationDepth.Infinity,
    }
    public partial class TargetItemViewModel: ViewModelBase
    {
        [ObservableProperty]
        public partial string Path { get; set; } = string.Empty;
    }

    [ObservableProperty] public partial ValidDepth Depth { get; set; } = ValidDepth.Infinity;
    
    public static Type DepthType { get; } = typeof(ValidDepth);


    public List<TargetItemViewModel> TargetItems => TargetEntries.Select(i => new TargetItemViewModel()
    {
        Path = i
    }).ToList();
    
    
    
    [ObservableProperty]
    public partial string Path { get; set; } = string.Empty;
    
    // public required string RelateTo { get; set; }
    
    public required List<string> TargetEntries { get; set; }
    
    [ObservableProperty]
    public partial bool IgnoreExternals { get; set; }
    
    [ObservableProperty]
    public partial bool AllowUnversionedObstructions { get; set; }
    
    [ObservableProperty]
    public partial bool AddAsModification { get; set; }
    
    [ObservableProperty]
    public partial bool MakeParents { get; set; }
    
    [ObservableProperty]
    public partial bool IsHead { get; set; } = true;

    [ObservableProperty]
    public partial bool IsNumber { get; set; }

    [ObservableProperty]
    public partial bool IsDate { get; set; }

    [ObservableProperty]
    public partial bool IsCommitted { get; set; }

    [ObservableProperty]
    public partial bool IsPrevious { get; set; }

    [ObservableProperty]
    public partial uint Number { get; set; }

    [ObservableProperty]
    public partial DateTime DateTime { get; set; } = DateTime.Now;

    protected override async Task OnConfirm()
    {
        Revision? revision = null;

        if (IsHead)
        {
            revision = new Revision.Head();
        } else if (IsNumber)
        {
            revision = new Revision.Number(Number);
        } else if (IsCommitted)
        {
            revision = new Revision.Committed();
        } else if (IsPrevious)
        {
            revision = new Revision.Previous();
        } else if (IsDate)
        {
            revision = new Revision.Date(DateTime.Second);
        }

        if (revision == null)
        {
            throw new UnreachableException();
        }


        var updateOptions = new UpdateOptions(TargetItems.Select(i => i.Path).ToArray(), revision, (OperationDepth)Depth, false, IgnoreExternals, AllowUnversionedObstructions, AddAsModification, MakeParents);

        var context = SendMessage(new OnGetContext()).Response;

        await context.Update(updateOptions);
        
        Ok();
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        IsCloseButtonVisible = false,
        Title = "Update",
        Buttons = DialogButton.None
    };
}