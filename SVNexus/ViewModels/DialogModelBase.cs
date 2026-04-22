using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;

namespace SVNexus.ViewModels;



public abstract partial class DialogModelBase(ViewModelBase? parent = null) : ViewModelBase(parent), IDialogContext
{

    public abstract Task OnConfirm();
    
    [RelayCommand]
    public async Task Confirm()
    {
        try
        {
            await OnConfirm();
        }
        finally
        {
            Accept = true;
            RequestClose?.Invoke(this, null);
        }
    }
    
    public bool Accept { get; set; }
    
    [RelayCommand]
    public void Close()
    {
        Accept = false;
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;
}