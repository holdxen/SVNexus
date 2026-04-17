using System;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class ImportProcessDialogModel(ViewModelBase parent) : ViewModelMore(parent) , IDialogContext
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

    
    public required InitializeRepositoryOptions Options { get; init; }

    protected override async Task LoadOnce()
    {
        var hostId = SendMessage(new OnGetDialogHostId());


        var context = EngineBackend.Instance.SimpleContext(hostId);
        
        try
        {

            await context.InitializeRepository(Options, new InitializeRepositoryNotifierDelegate
            {
                OnBackupAction = () =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        Steps.LastOrDefault()?.State =
                            StepState.Success;
                        Steps.Add(new StepItemViewModel()
                        {
                            Content = "Backup",
                            DateTime = DateTime.Now,
                            State = StepState.Loading,
                            Title = "Backup"
                        });
                    });
                },
                OnBackupFinishedAction = path =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        var last = Steps.LastOrDefault();
                        last?.State =
                            StepState.Success;
                        last?.DateTime = DateTime.Now;
                        last?.Content = $"Backup finished: {path}";
                    });
                },
                OnCheckoutAction = () =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        Steps.LastOrDefault()?.State =
                            StepState.Success;
                        Steps.Add(new StepItemViewModel()
                        {
                            Title = "Checkout",
                            Content = $"Checkout from {Options.Remote}",
                            State = StepState.Loading,
                            DateTime = DateTime.Now
                        });
                    });
                },
                OnCheckoutDirectlyAction = () =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        Steps.Add(new StepItemViewModel()
                        {
                            Title = "Checkout",
                            DateTime = DateTime.Now,
                            State = StepState.Loading,
                        });
                    });
                },
                OnFinishedAction = () =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        Steps.LastOrDefault()?.State =
                            StepState.Success;
                    });
                },
                OnImportAction = () =>
                {
                    Dispatcher.UIThread.Invoke(() =>
                    {
                        Steps.Add(new StepItemViewModel()
                        {
                            Title = "Import",
                            Content = "Import",
                            State = StepState.Loading,
                            DateTime = DateTime.Now
                        });
                    });
                }
            });
            IsCompleted = true;
        }
        catch (System.Exception e)
        {
            Console.WriteLine(e);
            var last = Steps.LastOrDefault();
            last?.State = StepState.Error;
            last?.Content = e.HumanReadableMessage;
            Error = e.HumanReadableMessage;
        }
        finally
        {
            context.Dispose();
            IsCanceling = false;
        }


    }
}