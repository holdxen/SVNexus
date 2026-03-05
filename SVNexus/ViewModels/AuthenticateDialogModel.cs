using System;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class AuthenticateDialogModel: ViewModelBase, IDialogContext
{
    public override Type? ViewType { get; set; } = typeof(AuthenticateDialog);
    
    
    [ObservableProperty]
    public partial string Username { get; set; } = string.Empty;
    
    [ObservableProperty]
    public partial string Password { get; set; } = string.Empty;


    [ObservableProperty] public partial string Realm { get; set; } = string.Empty;
    
    
    [ObservableProperty] public partial bool Savable { get; set; }
    
    [ObservableProperty] public partial bool Save { get; set; }
    
    public bool Accept { get; private set; }

    public void Close()
    {
        RequestClose?.Invoke(this, null);
        Accept = false;
    }

    public event EventHandler<object?>? RequestClose;


    [RelayCommand]
    private void Cancel()
    {
        Close();
    }

    [RelayCommand]
    private void Confirm()
    {
        RequestClose?.Invoke(this, null);
        Accept = true;
    }
}