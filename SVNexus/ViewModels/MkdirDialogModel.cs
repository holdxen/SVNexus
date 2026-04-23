using System;
using System.Collections.Generic;
using System.ComponentModel.DataAnnotations;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Validation;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class MkdirDialogModel(ViewModelBase parent): DialogModelBase(parent)// ViewModelBase, IDialogContext
{
    [ObservableProperty] public partial string ParentDirectory { get; set; } = string.Empty;

    [FolderNameValidation]
    public string Name
    {
        get;
        set => SetProperty(ref field, value);
    } = string.Empty;

    // public bool Accept { get; set; }

    protected override async Task OnConfirm()
    {
        if (!ValidateAllProperty(out _))
        {
            return;
        }

        try
        {
            var path = ParentDirectory + "/" + Name;
        
            var options = new MkdirOptions([path], true, null, string.Empty);
            var context = SendMessage(new OnGetContext()).Response;
            await context.Mkdir(options);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = "Failed to create directory: " + e.HumanReadableMessage,
            }, Manager.MainWindowToken);
        }
        
        Ok();
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Create a versioned directory",
        IsCloseButtonVisible = false,
        Buttons = DialogButton.None,
        Mode = DialogMode.Question
    };

    // [RelayCommand]
    // private void Confirm()
    // {
    //     if (!ValidateAllProperty(out _))
    //     {
    //         return;
    //     }
    //     RequestClose?.Invoke(this, null);
    //     Accept = true;
    // }
    //

    // [RelayCommand]
    // public void Close()
    // {
    //     RequestClose?.Invoke(this, null);
    //     Accept = false;
    // }

    // public event EventHandler<object?>? RequestClose;
}