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

    public partial class ProcessDetailItemViewModel: ViewModelBase
    {
    
        [ObservableProperty]
        private string _action = string.Empty;
    
    
        [ObservableProperty]
        private string _path = string.Empty;
    
    
        [ObservableProperty]
        private string _mimeType = string.Empty;
    }


    [ObservableProperty]
    private string _url = string.Empty;
    
    
    [ObservableProperty]
    private string _path = string.Empty;
    
    [ObservableProperty]
    private string _revision = string.Empty;
    
    
    [ObservableProperty]
    private string _username = string.Empty;
    
    [ObservableProperty]
    private string _startTime = string.Empty;
    
    
    [ObservableProperty]
    private string _currentFile = string.Empty;


    public ObservableCollection<ProcessDetailItemViewModel> ProcessDetailItems { get; set; } = [];


    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Closable))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(OpenButtonVisible))]
    private bool _isCompleted;

    public bool Closable => IsCompleted;


    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;


    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(HandleText))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    private long _total;
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsIndeterminate))]
    [NotifyPropertyChangedFor(nameof(HandleText))]
    [NotifyPropertyChangedFor(nameof(Process))]
    [NotifyPropertyChangedFor(nameof(ProcessText))]
    private long _downloaded;
    
    
    
    public bool IsIndeterminate => !IsCompleted && (Total < 0 || Downloaded < 0);

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
    [NotifyPropertyChangedFor(nameof(OpenButtonVisible))]
    private string? _error;

    public bool OpenButtonVisible => IsCompleted && Error is null;


    [RelayCommand]
    private void OpenRepository()
    {
        if (!OpenButtonVisible)
        {
            return;
        }

        WeakReferenceMessenger.Default.Send(new OnOpenRepository(Path));
        
        
        
        Close();
    }

}