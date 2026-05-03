using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class AuthenticateDialogModel: DialogModelBase
{
    [ObservableProperty]
    public partial string Username { get; set; } = string.Empty;
    
    [ObservableProperty]
    public partial string Password { get; set; } = string.Empty;


    [ObservableProperty] public partial string Realm { get; set; } = string.Empty;
    
    
    [ObservableProperty] public partial bool Savable { get; set; }
    
    [ObservableProperty] public partial bool Save { get; set; }

    [RelayCommand]
    private void Cancel()
    {
        Close();
    }

    protected override Task OnConfirm()
    {
        Ok();
        return Task.CompletedTask;
    }
}