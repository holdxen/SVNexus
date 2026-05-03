using System;
using System.Diagnostics;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Views;
using OperationDepth = SVNexus.Generated.Depth;


namespace SVNexus.ViewModels;



public partial class CheckoutOrExportDialogModel : ViewModelBase, IDialogContext
{
    // public required WeakReferenceMessenger Messenger { get; init; }
    
    public enum ValidDepth
    {
        Empty = OperationDepth.Empty,
        Files =  OperationDepth.Files,
        Immediates =  OperationDepth.Immediates,
        Infinity = OperationDepth.Infinity,
    }


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


    [ObservableProperty]
    public partial bool IsHead { get; set; } = true;

    [ObservableProperty]
    public partial bool IsNumber { get; set; }

    [ObservableProperty]
    public partial bool IsDate { get; set; }

    [ObservableProperty]
    public partial bool IsCommitted { get; set; }

    [ObservableProperty]
    public partial bool IsPrevious { get; set; }

    [ObservableProperty]
    public partial uint Number { get; set; }

    [ObservableProperty]
    public partial DateTime DateTime { get; set; } = DateTime.Now;

    [ObservableProperty]
    public partial ValidDepth Depth { get; set; } = ValidDepth.Infinity;

    public Type DepthType => typeof(ValidDepth);


    [ObservableProperty]
    public partial bool IgnoreExternal { get; set; }

    [ObservableProperty]
    public partial string Path { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string Url { get; set; } = string.Empty;
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
            Manager.Default.Send(new OnNotification
            {
                Title = "错误",
                Content = "Url must be specified.",
                Type = NotificationType.Error,
                ShowIcon = true,
            }, Manager.MainWindowToken);
            return;
        }

        var path = Path.Trim();
        
        if (string.IsNullOrEmpty(path))
        {
            Manager.Default.Send(new OnNotification
            {
                Title = "错误",
                Content = "Url must be specified.",
                Type = NotificationType.Error,
                ShowIcon = true,
            },  Manager.MainWindowToken);
            return;
        }


        Options = new CheckoutOptions(
            url,
            path,
            revision,
            revision,
            (OperationDepth)Depth,
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