using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using Ursa.Controls;

namespace SVNexus.ViewModels;



public abstract partial class DialogModelBase(ViewModelBase? parent = null) : ViewModelBase(parent), IDialogContext
{

    public void Ok()
    {
        Accept = true;
        RequestClose?.Invoke(this, null);
    }

    protected abstract Task OnConfirm();
    
    [RelayCommand]
    private async Task Confirm()
    {
        await OnConfirm();
    }
    
    public bool Accept { get; set; }
    
    [RelayCommand]
    public void Close()
    {
        Accept = false;
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;
    
    
    public virtual OverlayDialogOptions OverlayDialogOptions { get; } = new();
}