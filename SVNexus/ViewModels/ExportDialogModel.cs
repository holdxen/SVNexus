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
using Notification = Ursa.Controls.Notification;


namespace SVNexus.ViewModels;

public partial class ExportDialogModel: ViewModelBase, IDialogContext
{
    
    public static Type DepthType => typeof(Depth);
    
    public static Type NativeEolType => typeof(NativeEol);

    [ObservableProperty] public partial string Url { get; set; } = string.Empty;
    
    [ObservableProperty] public partial string Path { get; set; } = string.Empty;


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
    public partial Depth Depth { get; set; } = Depth.Infinity;



    [ObservableProperty]
    public partial bool IgnoreExternals { get; set; }
    
    [ObservableProperty]
    public partial bool IgnoreKeywords { get; set; }
    
    [ObservableProperty]
    public partial bool Override { get; set; }


    [ObservableProperty] public partial NativeEol NativeEol { get; set; } = NativeEol.None;
    
    
    // public required WeakReferenceMessenger Messenger { get; init; }

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

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;

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
            Manager.Default.Send(new OnNotification(new Notification
            {
                Title = "错误",
                Content = "Url must be specified.",
                Type = NotificationType.Error,
                ShowIcon = true,
            }), Manager.MainWindowToken);
            return;
        }

        var options = new ExportOptions(
            FromPathOrUrl: url,
            ToPath: path,
            PegRevision: revision,
            Revision: revision,
            Override: Override,
            IgnoreExternals: IgnoreExternals,
            IgnoreKeywords: IgnoreKeywords,
            Depth: Depth,
            NativeEol: NativeEol);
        
        Manager.Default.Send(new OnExport(options), Token);
        
        Close();
    }
}