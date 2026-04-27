using System;
using System.Collections.ObjectModel;
using System.ComponentModel.Design;
using System.Globalization;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class CheckoutProcessDialogModel(ViewModelBase parent) : ProcessDialogModel(parent)
{
    
    public required CheckoutOptions Options { get; set; }
    
    private ContextNotifierDelegate? _contextNotifier;
    
    protected override async Task OnStart()
    {
        var hostId = SendMessage(new OnGetDialogHostId());
        
        Url = Options.Url;
        Path = Options.Path;


        _contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStartString(Path);
                    
                    ProcessLogItems.Add(new ProcessLogItemViewModel()
                    {
                        Action = notify.Action.ToString(),
                        MimeType =  notify.MimeType ?? "",
                        Path = path
                    });
                    CurrentFile = path;
                });
            },
            ProgressNotifyAction = (downloaded, total) =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    Total = total;
                    Downloaded = downloaded;
                });
            },
            DialogHostId = hostId
        };
        
        var createContextOptions = EngineBackend.Instance.MakeCreateContextOptions(_contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        try
        {
        
            var number = await context.Checkout(Options);
            Revision = number.ToString();
            IsCompleted = true;
        }
        catch (System.Exception e)
        {
            Error = e.HumanReadableMessage;
        }
        finally
        {
            context.Dispose();
            IsCanceling = false;
            _contextNotifier = null;
        }

    }

    protected override void OnCancel()
    {
        if (_contextNotifier is null)
        {
            return;
        }
        IsCanceling = true;
        _contextNotifier.CancelMessage = "User cancelled";
        _contextNotifier = null;
    }
}

public partial class ExportProcessDialogModel(ViewModelBase parent) : ProcessDialogModel(parent)
{
    
    public required ExportOptions Options { get; set; }
    
    private ContextNotifierDelegate? _contextNotifier;
    
    protected override async Task OnStart()
    {
        var hostId = SendMessage(new OnGetDialogHostId());
        
        Url = Options.FromPathOrUrl;
        Path = Options.ToPath;


        _contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStartString(Path);
                    
                    ProcessLogItems.Add(new ProcessLogItemViewModel()
                    {
                        Action = notify.Action.ToString(),
                        MimeType =  notify.MimeType ?? "",
                        Path = path
                    });
                    CurrentFile = path;
                });
            },
            ProgressNotifyAction = (downloaded, total) =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    Total = total;
                    Downloaded = downloaded;
                });
            },
            DialogHostId = hostId
        };
        
        var createContextOptions = EngineBackend.Instance.MakeCreateContextOptions(_contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        try
        {
        
            var number = await context.Export(Options);
            Revision = number.ToString();
            IsCompleted = true;
        }
        catch (System.Exception e)
        {
            Error = e.HumanReadableMessage;
        }
        finally
        {
            context.Dispose();
            IsCanceling = false;
            _contextNotifier = null;
        }

    }

    protected override void OnCancel()
    {
        if (_contextNotifier is null)
        {
            return;
        }
        IsCanceling = true;
        _contextNotifier.CancelMessage = "User cancelled";
        _contextNotifier = null;
    }
}

public abstract partial class ProcessDialogModel(ViewModelBase parent): ViewModelMore(parent), IDialogContext
{
    public override Type? ViewType { get; set; } = typeof(CheckoutOrExportProcessDialog);

    public partial class ProcessLogItemViewModel: ViewModelBase
    {

        [ObservableProperty]
        public partial string Action { get; set; } = string.Empty;

        [ObservableProperty]
        public partial string Path { get; set; } = string.Empty;

        [ObservableProperty]
        public partial string MimeType { get; set; } = string.Empty;
    }


    [ObservableProperty]
    public partial string Url { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string Path { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string Revision { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string Username { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string StartTime { get; set; } = string.Empty;

    [ObservableProperty]
    public partial string CurrentFile { get; set; } = string.Empty;

    public ObservableCollection<ProcessLogItemViewModel> ProcessLogItems { get; set; } = [];


    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Closable))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(OpenButtonVisible))]
    [NotifyPropertyChangedFor(nameof(CancelButtonVisible))]
    public partial bool IsCompleted { get; set; }

    public bool Closable => IsCompleted || Error is not null;
    
    
    // public required WeakReferenceMessenger Messenger { get; init; }


    [RelayCommand]
    public void Close()
    {
        if (IsCompleted || Error is not null)
        {
            RequestClose?.Invoke(this, null);
        }
        // else
        // {
        //     Messenger.Send(new OnCancel()
        //     {
        //         Model = this
        //     });
        // }
    }

    [RelayCommand]
    public void Shutdown()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;


    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(HandleText))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    public partial long Total { get; set; }

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(HandleText))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    public partial long Downloaded { get; set; }

    public bool IsIndeterminate => !IsCompleted && Error is null && (Total < 0 || Downloaded < 0);
    // {
    //     get
    //     {
    //         var value = !IsCompleted && Error is null && (Total < 0 || Downloaded < 0);
    //         Console.WriteLine("IsIndeterminate: " + value);
    //         Console.WriteLine("IsCompleted: " + IsCompleted);
    //         Console.WriteLine("Error: " + Error);
    //         Console.WriteLine("Total: " + Total);
    //         Console.WriteLine("Downloaded: " + Downloaded);
    //         return value;
    //     }
    // }

    public string ProcessText => Process.ToString(CultureInfo.InvariantCulture) + "%";

    public double Process
    {
        get
        {
            if (IsCompleted)
            {
                return 100;
            }
            if (IsIndeterminate)
            {
                return 0;
            }

            return (double)Downloaded / Total * 100.0;
        }
    }


    public string HandleText =>
        $"{(Downloaded > 0 ? Downloaded.ToString() : "Unknown")}/{(Total > 0 ? Total.ToString() : "Unknown")}";


    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Closable))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(OpenButtonVisible))]
    [NotifyPropertyChangedFor(nameof(CancelButtonVisible))]
    public partial string? Error { get; set; }

    public bool OpenButtonVisible => IsCompleted && Error is null;
    
    
    public bool CancelButtonVisible => Error is null && !IsCompleted;
    
    [ObservableProperty]
    public partial bool IsCanceling { get; set; }

    protected abstract Task OnStart();
    
    protected abstract void OnCancel();


    protected override Task LoadOnce()
    {
        return OnStart();
    }

    [RelayCommand]
    private void Cancel()
    {
        OnCancel();
        // if (IsCanceling)
        // {
        //     return;
        // }
        //
        // // Messenger.Send(new OnCancel());
        // SendMessage(new OnCancel());
        // IsCanceling = true;
    }


    [RelayCommand]
    private void OpenRepository()
    {
        if (!OpenButtonVisible)
        {
            return;
        }

        Manager.Default.Send(new OnOpenRepository(Path), Manager.MainWindowToken);
        
        
        Close();
    }

}