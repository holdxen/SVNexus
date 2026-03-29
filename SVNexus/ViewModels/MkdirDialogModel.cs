using System;
using System.ComponentModel.DataAnnotations;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Messages;
using SVNexus.Validation;

namespace SVNexus.ViewModels;

public partial class MkdirDialogModel: ObservableValidator, IDialogContext
{
    [ObservableProperty] public partial string ParentDirectory { get; set; } = string.Empty;

    // [ObservableProperty]
    // [FolderNameValidation]
    // [NotifyDataErrorInfo]
    // [Required]
    // public partial string Name { get; set; } = string.Empty;

    [MinLength(10)]

    public string Name
    {
        get => field;
        set => SetProperty(ref field, value);
    } = string.Empty;
    
    public bool Accept { get; set; }

    [RelayCommand]
    private void Confirm()
    {
        RequestClose?.Invoke(this, null);
        Accept = true;
    }
    
    
    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
        Accept = false;
    }

    public event EventHandler<object?>? RequestClose;
}