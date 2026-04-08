using System;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Messages;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class ImportProcessDialogModel : ViewModelBase, IDialogContext
{
    public override Type? ViewType { get; set; } = typeof(ImportProcessDialog);

    public enum StepState
    {
        Success,
        Error,
        Loading
    }


    public partial class StepItemViewModel : ViewModelLite
    {
        [ObservableProperty] public partial string Content { get; set; } = string.Empty;

        [ObservableProperty] public partial string Title { get; set; } = string.Empty;


        [ObservableProperty] public partial DateTime DateTime { get; set; }

        // [ObservableProperty] public partial bool IsFinished { get; set; }
        [ObservableProperty] public partial StepState State { get; set; } = StepState.Success;
    }


    public ObservableCollection<StepItemViewModel> Steps { get; } = [];


    [ObservableProperty]
    public partial bool IsCanceling { get; set; }
    
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(CancelButtonVisible))]
    [NotifyPropertyChangedFor(nameof(Closable))]
    public partial string? Error { get; set; }
    
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(CancelButtonVisible))]
    [NotifyPropertyChangedFor(nameof(Closable))]
    public partial bool IsCompleted { get; set; }
    
    
    public bool Closable => IsCompleted || Error is not null;
    
    public bool CancelButtonVisible => !IsCompleted && Error is null;
    
    public required WeakReferenceMessenger Messenger { get; init; }


    [RelayCommand]
    private void Cancel()
    {
        if (IsCanceling)
        {
            return;
        }

        // Messenger.Send(new OnCancel());
        IsCanceling = true;
    }
    
    [RelayCommand]
    public void Close()
    {
        if (Closable)
        {
            RequestClose?.Invoke(this, null);
        }
    }

    public event EventHandler<object?>? RequestClose;
}