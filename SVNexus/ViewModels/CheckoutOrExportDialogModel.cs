using System;
using System.Diagnostics;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Views;
using Notification = Ursa.Controls.Notification;


namespace SVNexus.ViewModels;



public partial class CheckoutOrExportDialogModel : ViewModelBase, IDialogContext
{
    // public required WeakReferenceMessenger Messenger { get; init; }

    public override Type? ViewType { get; set; } = typeof(CheckoutOrExportDialog);


    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;
    
    
    // [ObservableProperty]
    // [NotifyPropertyChangedFor(nameof(IsHead))]
    // private Revision _revision = new Revision.Head();
    //
    //
    //
    // public bool IsHead => Revision is Revision.Head;
    //
    // public bool IsNumber => Revision is Revision.Number;
    //
    //
    // public bool IsDate => Revision is Revision.Date;
    //
    // public bool IsCommitted => Revision is Revision.Committed;
    //
    // public bool IsPrevious => Revision is Revision.Previous;


    [ObservableProperty] private bool _isHead = true;

    [ObservableProperty] private bool _isNumber;
    
    [ObservableProperty] private bool _isDate;
    
    [ObservableProperty] private bool _isCommitted;
    
    [ObservableProperty] private bool _isPrevious;
    
    
    
    [ObservableProperty] private uint _number;


    [ObservableProperty] private DateTime _dateTime = DateTime.Now;


    [ObservableProperty] private Depth _depth = Depth.Infinity;
    
    
    
    public Type DepthType => typeof(Depth);


    [ObservableProperty]
    private bool _ignoreExternal;


    [ObservableProperty] private string _path = string.Empty;

    [ObservableProperty] private string _url = string.Empty;

    public CheckoutOptions? Options { get; set; }

    [RelayCommand]
    private void Confirm()
    {
        Revision? revision = null;

        if (IsHead)
        {
            revision = new Revision.Head();
        } else if (IsNumber)
        {
            revision = new Revision.Number(Number);
        } else if (IsCommitted)
        {
            revision = new Revision.Committed();
        } else if (IsPrevious)
        {
            revision = new Revision.Previous();
        } else if (IsDate)
        {
            revision = new Revision.Date(DateTime.Second);
        }

        if (revision == null)
        {
            throw new UnreachableException();
        }

        var url = Url.Trim();

        if (string.IsNullOrEmpty(url))
        {
            Manager.Default.Send(new OnNotification(new Notification()
            {
                Title = "错误",
                Content = "Url must be specified.",
                Type = NotificationType.Error,
                ShowIcon = true,
            }), Manager.MainWindowToken);
            return;
        }

        var path = Path.Trim();
        
        if (string.IsNullOrEmpty(path))
        {
            Manager.Default.Send(new OnNotification(new Notification()
            {
                Title = "错误",
                Content = "Url must be specified.",
                Type = NotificationType.Error,
                ShowIcon = true,
            }),  Manager.MainWindowToken);
            return;
        }


        Options = new CheckoutOptions(
            url,
            path,
            revision,
            revision,
            Depth,
            IgnoreExternal,
            false,
            null
            );
        
        
        
        // Manager.Default.Send(new OnCheckout(checkoutOptions), Token);

        Close();

    }


    [RelayCommand]
    private async Task SelectFolder()
    {
        var options = new FolderPickerOpenOptions()
        {
            AllowMultiple = false,
            Title = "Select a folder to checkout",
        };
        
        
        var result = await Manager.Default.Send(new OnFolderPickerOpen(options), Manager.MainWindowToken);
        if (result.Count > 0)
        {
            Path = result[0].Path.AbsolutePath;
        }
    }
}