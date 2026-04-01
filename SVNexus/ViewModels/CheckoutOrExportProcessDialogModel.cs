using System;
using System.Collections.ObjectModel;
using System.Globalization;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Messages;
using SVNexus.ViewModels;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class CheckoutOrExportProcessDialogModel: ViewModelBase, IDialogContext
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
    
    
    public required WeakReferenceMessenger Messenger { get; init; }


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


    [RelayCommand]
    private void Cancel()
    {
        if (IsCanceling)
        {
            return;
        }

        Messenger.Send(new OnCancel());
        IsCanceling = true;
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