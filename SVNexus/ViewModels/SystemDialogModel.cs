using CommunityToolkit.Mvvm.ComponentModel;

namespace SVNexus.ViewModels;

public partial class SystemDialogModel(ViewModelBase? parent = null): ViewModelBase(parent)
{
    [ObservableProperty]
    public partial object? Content { get; set; }
    
    
    [ObservableProperty]
    public partial string? Title { get; set; }
}