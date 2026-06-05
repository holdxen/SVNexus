using System.Collections.ObjectModel;
using System.Threading.Tasks;
using Avalonia.Controls.Primitives;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Components;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using Ursa.Controls;
using Exception = SVNexus.Generated.Exception;

namespace SVNexus.ViewModels;

public partial class InitializeRepositoryProcessDialogModel(ViewModelBase parent): DialogModelBase(parent)
{

    public class LogViewModel: ViewModelBase
    {
        public required WorkingCopyNotify Notify { get; set; }

        public string ActionText => Notify.Action.ToString();
        
        public string Path => Notify.Path;

        public string FileName => Notify.Path.GetFileName();
        
        public string Directory => Notify.Path.GetDirectoryName()?.TrimStartString(RelateTo).TrimStartString("/") ?? string.Empty;
        
        public string RelateTo { get; set; } = string.Empty;
    }
    
    public required InitializeRepositoryOptions InitializeRepositoryOptions { get; init; }

    [ObservableProperty]
    public partial LoadingDot.State ImportState { get; set; }  = LoadingDot.State.Running;

    [ObservableProperty] public partial string ImportDetail { get; set; } = "Waiting ...";
    
    [ObservableProperty]
    public partial LoadingDot.State BackupState { get; set; }  = LoadingDot.State.Ready;
    
    [ObservableProperty] public partial string BackupDetail { get; set; } = "Waiting ...";
    
    [ObservableProperty] public partial string? BackupFile { get; set; }
    
    
    [ObservableProperty]
    public partial LoadingDot.State CheckoutState { get; set; }  = LoadingDot.State.Ready;
    
    [ObservableProperty] public partial string CheckoutDetail { get; set; } = "Waiting ...";

    public ObservableCollection<LogViewModel> LogViewModels { get; } = [];

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Import and checkout",
        IsCloseButtonVisible = false,
        CanResize = true,
        Buttons = DialogButton.None,
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        VerticalScrollBarVisibility = ScrollBarVisibility.Disabled,
        CanDragMove = true
    };

    [ObservableProperty]
    public partial bool Retry { get; set; }

    [ObservableProperty] public partial int ScrollTo { get; set; } = -1;

    private TaskCompletionSource? _cancelSource;
    
    private LockedValue<string?>? _cancelMessage;


    [RelayCommand]
    private void DoRetry()
    {
        Retry = true;
        RequestToClose(null);
    }
    
    protected override Task OnConfirm()
    {
        Accept = false;
        RequestToClose(null);
        return Task.CompletedTask;
    }

    // [RelayCommand]
    // private async Task OnLoaded()
    // {
    //     BackupFile = "test";
    // }

    private void AddLogs(WorkingCopyNotify notify)
    {
        const int max = 1024;
        LogViewModels.Add(new LogViewModel()
        {
            Notify = notify,
            RelateTo = InitializeRepositoryOptions.Local.TrimEndPathSeparatorChar()
        });

        if (LogViewModels.Count > max)
        {
            LogViewModels.RemoveAt(0);
        }
        ScrollTo = LogViewModels.Count - 1;
    }
    
    [RelayCommand]
    private async Task CopyBackupFilePath()
    {
        await Manager.Default.Send(new ClipBoardMessages.SetText()
        {
            Text = BackupFile ?? string.Empty,
        }, Manager.MainWindowToken);
    }

    [RelayCommand]
    private async Task OnLoaded()
    {
        var notifier = new InitializeRepositoryNotifierDelegate
        {
            Dispatch = true,
            OnCheckoutDirectlyAction = () =>
            {
                ImportState = LoadingDot.State.Success;
                ImportDetail = "Skip";
                
                BackupState = LoadingDot.State.Success;
                BackupDetail = "Skip";
                
                CheckoutState = LoadingDot.State.Running;
                CheckoutDetail = "Running ...";
            },
            OnImportAction = () =>
            {
                ImportState = LoadingDot.State.Running;
                ImportDetail = "Running ...";
            },
            OnBackupAction = () =>
            {
                ImportState = LoadingDot.State.Success;
                ImportDetail = "Done";
                
                BackupState = LoadingDot.State.Running;
                BackupDetail = "Running ...";
            },
            OnBackupFinishedAction = file =>
            {
                ImportState = LoadingDot.State.Success;
                ImportDetail = "Done";
                
                BackupState = LoadingDot.State.Success;
                BackupDetail = "Done ..";
                
                BackupFile = file;
            },
            OnCheckoutAction = () =>
            {
                ImportState = LoadingDot.State.Success;
                ImportDetail = "Done";
                
                BackupState = LoadingDot.State.Success;
                BackupDetail = "Done";
                
                CheckoutState = LoadingDot.State.Running;
                CheckoutDetail = "Running ...";
            },
            OnFinishedAction = () =>
            {
                ImportState = LoadingDot.State.Success;
                BackupState = LoadingDot.State.Success;
                
                CheckoutState = LoadingDot.State.Success;
                CheckoutDetail = "Done";
            }
        };

        // var context = SendMessage(new OnGetContext()).Response;
        //
        var contextNotifier = new ContextNotifierDelegate()
        {
            DialogHostId = SendMessage(new OnGetDialogHostId()),
            CancelMessage =  _cancelMessage ??= new LockedValue<string?>(null),
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    HandleNotify(notify);
                });
            }
        };
        
        
        var context = EngineBackend.Instance.SimpleContext(contextNotifier);
        
        
        try
        {
            await context.InitializeRepository(InitializeRepositoryOptions, notifier);

            Accept = true;
        }
        catch (Exception e)
        {
            if (ImportState == LoadingDot.State.Running)
            {
                ImportState = LoadingDot.State.Failure;
                ImportDetail = e.HumanReadableMessage;
            } 
            else if (CheckoutState == LoadingDot.State.Running)
            {
                CheckoutState = LoadingDot.State.Failure;
                CheckoutDetail = e.HumanReadableMessage;
            }
            else if (BackupState == LoadingDot.State.Success)
            {
                BackupState = LoadingDot.State.Failure;
                BackupDetail = e.HumanReadableMessage;
            }
            else
            {
                ImportState = LoadingDot.State.Failure;
                ImportDetail = e.HumanReadableMessage;
            }

            Accept = false;
            
            Logger.Info($"Failed to initialize repository: {e.HumanReadableMessage}\n{e.StackTrace}");
        }
        finally
        {
            _cancelSource?.SetResult();
        }
    }

    private void HandleNotify(WorkingCopyNotify notify)
    {
        AddLogs(notify);
        if (ImportState == LoadingDot.State.Running)
        {
            ImportDetail = $"{notify.Action}:{notify.Path}";
        } 
        else if (CheckoutState == LoadingDot.State.Running)
        {
            CheckoutDetail = $"{notify.Action}:{notify.Path}";
        }
        else if (BackupState == LoadingDot.State.Success)
        {
            BackupDetail = $"{notify.Action}:{notify.Path}";
        }
    }
    
    [RelayCommand]
    private async Task Cancel()
    {
        var hostId = SendMessage(new OnGetDialogHostId());

        var result = await OverlayMessageBox.ShowAsync("Whether to cancel", "Question", hostId, MessageBoxIcon.Question, MessageBoxButton.YesNo);
        if (result != MessageBoxResult.Yes)
        {
            return;
        }
        
        _cancelSource = new TaskCompletionSource();

        _cancelMessage?.Value = "User cancelled";
        
        await _cancelSource.Task;
        _cancelSource = null;
    }
}