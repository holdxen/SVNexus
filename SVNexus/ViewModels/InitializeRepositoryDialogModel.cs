using System.IO;
using System.Threading.Tasks;
using Avalonia.Controls.Primitives;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class InitializeRepositoryDialogModel: DialogModelBase
{
    protected override async Task OnConfirm()
    {
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Initialize",
        IsCloseButtonVisible = false,
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        VerticalScrollBarVisibility = ScrollBarVisibility.Disabled,
        Buttons = DialogButton.None,
    };


    [ObservableProperty]
    public partial string Local { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string Remote { get; set; } = string.Empty;


    [ObservableProperty] public partial string BackupIn { get; set; } = string.Empty;

    [ObservableProperty] public partial string CommitMessage { get; set; } = string.Empty;
    
    [ObservableProperty] public partial bool IgnoreUnknownNodeTypes { get; set; }
    
    [ObservableProperty] public partial bool NoIgnore { get; set; }
    
    [ObservableProperty] public partial bool NoAutoProperties { get; set; }


    [ObservableProperty] public partial bool Backup { get; set; } = true;

    [ObservableProperty] public partial string Ignore { get; set; } = string.Empty;


    [RelayCommand]
    private void OnLoaded()
    {
        BackupIn = Directory.CreateTempSubdirectory().FullName;
    }
}