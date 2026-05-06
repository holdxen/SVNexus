using System;
using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls.Primitives;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Components;
using SVNexus.Generated;
using SVNexus.Messages;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class PatchDialogModel(ViewModelBase parent): DialogModelBase(parent)
{
    [ObservableProperty]
    public required partial string Path { get; set; }

    [ObservableProperty] public required partial string Target { get; set; }


    [ObservableProperty] public partial LoadingOrErrorState State { get; set; } = LoadingOrErrorState.MakeNone();

    [ObservableProperty] public partial string PatchContent { get; set; } = string.Empty;
    
    
    [ObservableProperty] public partial uint Strip { get; set; }
    
    [ObservableProperty] public partial bool IgnoreWhitespace { get; set; }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        FullScreen = true,
        Title = "Patch",
        IsCloseButtonVisible = false,
        StyleClass = "Fixed",
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        VerticalScrollBarVisibility = ScrollBarVisibility.Disabled,
    };


    
    [RelayCommand]
    private async Task Loaded()
    {
        PatchContent = await File.ReadAllTextAsync(Path);
    }

    protected override async Task OnConfirm()
    {
        var context = SendMessage(new OnGetContext()).Response;

        var patchOptions = new PatchOptions(Path, Target, false, Strip, false, IgnoreWhitespace, true);
        
        await context.Patch(patchOptions);
        
        Ok();
    }
}