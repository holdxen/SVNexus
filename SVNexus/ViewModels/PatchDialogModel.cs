using System;
using System.IO;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Components;
using SVNexus.Generated;
using SVNexus.Messages;

namespace SVNexus.ViewModels;

public partial class PatchDialogModel(ViewModelBase parent): ViewModelBase(parent), IDialogContext
{
    [ObservableProperty]
    public required partial string Path { get; set; }

    [ObservableProperty] public required partial string Target { get; set; }


    [ObservableProperty] public partial LoadingOrErrorState State { get; set; } = LoadingOrErrorState.MakeNone();

    [ObservableProperty] public partial string PatchContent { get; set; } = string.Empty;
    
    
    [ObservableProperty] public partial uint Strip { get; set; }
    
    [ObservableProperty] public partial bool IgnoreWhitespace { get; set; }
    
    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;



    [RelayCommand]
    private async Task Apply()
    {
        var context = SendMessage(new OnGetContext()).Response;

        var patchOptions = new PatchOptions(Path, Target, false, Strip, false, IgnoreWhitespace, true);
        
        await context.Patch(patchOptions);
        
        RequestClose?.Invoke(this, null);
    }

    
    [RelayCommand]
    private async Task Loaded()
    {
        PatchContent = await File.ReadAllTextAsync(Path);
    }
}