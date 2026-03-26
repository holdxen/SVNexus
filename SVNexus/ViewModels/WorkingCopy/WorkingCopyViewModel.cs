using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Microsoft.Extensions.DependencyInjection;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy.Changes;
using SVNexus.ViewModels.WorkingCopy.History;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.Views;
using Tmds.DBus.Protocol;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkingCopyViewModel : ViewModelLite,
    IRecipient<OnWorkingCopyViewEnabled>, 
    IRecipient<OnNotWorkingCopy>
{

    public const int LocalViewIndex = 0;
    public const int ChangesViewIndex = 1;
    public const int HistoryViewIndex = 2;
    public const int RemoteViewIndex = 3;
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsLocalView))]
    [NotifyPropertyChangedFor(nameof(IsRemoteView))]
    [NotifyPropertyChangedFor(nameof(IsChangesView))]
    [NotifyPropertyChangedFor(nameof(IsHistoryView))]
    public partial int SelectedViewIndex { get; set; } = ChangesViewIndex;

    // [ObservableProperty]
    // public required partial string WorkingCopyPath { get; set; }
    
    public bool IsLocalView => SelectedViewIndex == LocalViewIndex;

    public bool IsRemoteView => SelectedViewIndex == RemoteViewIndex;
    
    public bool IsHistoryView => SelectedViewIndex == HistoryViewIndex;
    
    public bool IsChangesView => SelectedViewIndex == ChangesViewIndex;
    
    
    public ChangesViewModel ChangesViewModel { get; }

    public HistoryViewModel HistoryViewModel { get; }
    
    
    public LocalViewModel LocalViewModel { get; } 
    
    [ObservableProperty] public partial bool IsEnabled { get; set; } = true;


    [RelayCommand]
    private async Task Update()
    {
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), _typeService.Get(this));
        try
        {
            using var context = Engine.Engine.Instance.SimpleContext(hostId);
            
            var opts = new UpdateOptions(Paths: [_workingCopyViewService.WorkingCopyPath], Revision: new Revision.Head(), Depth.Infinity, DepthIsSticky: false, IgnoreExternals: false, AllowUnverObstructions: true, AddsAdModification: false, MakeParents: true);
            
            await context.Update(opts);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to update working copy: {e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }

    private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    private readonly Services.ITabService  _tabService;
    private readonly Services.TypeService _typeService;

    public WorkingCopyViewModel(IServiceProvider serviceProvider)
    {
        _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
        _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
        _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
        
        LocalViewModel = serviceProvider.GetRequiredService<LocalViewModel>();
        
        ChangesViewModel = serviceProvider.GetRequiredService<ChangesViewModel>();
        
        HistoryViewModel = serviceProvider.GetRequiredService<HistoryViewModel>();
        
        Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
    }

    public void Receive(OnWorkingCopyViewEnabled message)
    {
        IsEnabled = message.Value;
    }

    public void Receive(InitializeRepositoryOptions message)
    {
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var contextNotifierDelegate = new ContextNotifierDelegate
        {
            DialogHostId = hostId,
        };

        var messenger = new WeakReferenceMessenger();

        var importProcessDialogModel = new ImportProcessDialogModel()
        {
            Messenger = messenger
        };
        
        messenger.Register<OnCancel>(contextNotifierDelegate, (recipient, cancel) =>
        {
            (recipient as ContextNotifierDelegate)!.CancelMessage = "User cancel";
        });


        
        Task.Run(async () =>
        {
            var context = AsyncContext.Create(Engine.Engine.Instance.MakeCreateContextOptions(contextNotifierDelegate));

            try
            {

                await context.InitializeRepository(message, new InitializeRepositoryNotifierDelegate
                {
                    OnBackupAction = () =>
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            importProcessDialogModel.Steps.LastOrDefault()?.State =
                                ImportProcessDialogModel.StepState.Success;
                            importProcessDialogModel.Steps.Add(new ImportProcessDialogModel.StepItemViewModel()
                            {
                                Content = "Backup",
                                DateTime = DateTime.Now,
                                State = ImportProcessDialogModel.StepState.Loading,
                                Title = "Backup"
                            });
                        });
                    },
                    OnBackupFinishedAction = path =>
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            var last = importProcessDialogModel.Steps.LastOrDefault();
                            last?.State =
                                ImportProcessDialogModel.StepState.Success;
                            last?.DateTime = DateTime.Now;
                            last?.Content = $"Backup finished: {path}";
                        });
                    },
                    OnCheckoutAction = () =>
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            importProcessDialogModel.Steps.LastOrDefault()?.State =
                                ImportProcessDialogModel.StepState.Success;
                            importProcessDialogModel.Steps.Add(new ImportProcessDialogModel.StepItemViewModel()
                            {
                                Title = "Checkout",
                                Content = $"Checkout from {message.Remote}",
                                State = ImportProcessDialogModel.StepState.Loading,
                                DateTime = DateTime.Now
                            });
                        });
                    },
                    OnCheckoutDirectlyAction = () =>
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            importProcessDialogModel.Steps.Add(new ImportProcessDialogModel.StepItemViewModel()
                            {
                                Title = "Checkout",
                                DateTime = DateTime.Now,
                                State = ImportProcessDialogModel.StepState.Loading,
                            });
                        });
                    },
                    OnFinishedAction = () =>
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            importProcessDialogModel.Steps.LastOrDefault()?.State =
                                ImportProcessDialogModel.StepState.Success;
                        });
                    },
                    OnImportAction = () =>
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            importProcessDialogModel.Steps.Add(new ImportProcessDialogModel.StepItemViewModel()
                            {
                                Title = "Import",
                                Content = "Import",
                                State = ImportProcessDialogModel.StepState.Loading,
                                DateTime = DateTime.Now
                            });
                        });
                    }
                });
                Dispatcher.UIThread.Invoke(() =>
                {
                    importProcessDialogModel.IsCompleted = true;
                });
            }
            catch (System.Exception e)
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var last = importProcessDialogModel.Steps.LastOrDefault();
                    last?.State = ImportProcessDialogModel.StepState.Error;
                    last?.Content = e.HumanReadableMessage;
                    importProcessDialogModel.Error = e.HumanReadableMessage;
                });
            }
            finally
            {
                context.Dispose();
            }
        });


        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Initialize repository",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };

            await OverlayDialog.ShowModal<ImportProcessDialog, ImportProcessDialogModel>(importProcessDialogModel, options: options, hostId: hostId);
            if (importProcessDialogModel.Error is null)
            {
                await ChangesViewModel.Status();
            }
        });

    }

    public void Receive(OnNotWorkingCopy message)
    {
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var result = await MessageBox.ShowOverlayAsync(title: "Error", hostId: hostId, message: "Not a working copy, initialize now", button: MessageBoxButton.YesNo);
            if (result is MessageBoxResult.No)
            {
                // Manager.MainWindow.Send(new OnRemoveTabByLocalViewModel(this));
                Manager.Default.Send(new OnRemoveTab(_tabService.Token), Manager.MainWindowToken);
            }
            else
            {
                var dialogOptions = new OverlayDialogOptions
                {
                    Title = "Test",
                    IsCloseButtonVisible = true,
                    Buttons = DialogButton.None
                };
                var importDialogModel = new ImportDialogModel()
                {
                    Path = _workingCopyViewService.WorkingCopyPath,
                };
                await OverlayDialog.ShowModal<ImportDialog, ImportDialogModel>(importDialogModel, hostId: hostId, options: dialogOptions);
                if (importDialogModel.Options is not null)
                {
                    Receive(importDialogModel.Options);
                }
            }

        });
    }
}