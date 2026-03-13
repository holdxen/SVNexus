using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.ViewModels.WorkingCopy.Remote;
using SVNexus.Views;
using Tmds.DBus.Protocol;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkingCopyViewModel : ViewModelBase, 
    IRecipient<OnGetDialogHostId>, 
    IRecipient<OnWorkingCopyViewEnabled>, 
    IRecipient<OnInitializeRepository>, 
    IRecipient<OnNotWorkingCopy>
{

    public const int LocalViewIndex = 0;
    public const int RemoteViewIndex = 1;
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsLocalView))]
    [NotifyPropertyChangedFor(nameof(IsRemoteView))]
    public partial int SelectedViewIndex { get; set; } = LocalViewIndex;
    
    public override bool KeepAlive { get; set; } = true;

    [ObservableProperty]
    public required partial string WorkingCopyPath { get; set; }
    
    public bool IsLocalView => SelectedViewIndex == LocalViewIndex;

    public bool IsRemoteView => SelectedViewIndex == RemoteViewIndex;
    
    
    [ObservableProperty]
    public partial LocalViewModel LocalViewModel { get; set; } = new();


    [ObservableProperty] public partial RemoteViewModel RemoteViewModel { get; set; } = new();

    
    [ObservableProperty]
    public partial string? DialogHostId { get; set; }

    // public required WeakReferenceMessenger Messenger { get; init; }


    [ObservableProperty] public partial bool IsEnabled { get; set; } = true;


    [RelayCommand]
    private async Task Update()
    {
        try
        {
            using var context = Engine.Engine.Instance.SimpleContext(DialogHostId);
            
            var opts = new UpdateOptions(Paths: [WorkingCopyPath], Revision: new Revision.Head(), Depth.Infinity, DepthIsSticky: false, IgnoreExternals: false, AllowUnverObstructions: true, AddsAdModification: false, MakeParents: true);
            
            await context.Update(opts);
        }
        catch (System.Exception e)
        {
            Console.WriteLine(e);
            throw;
        }
    }
    

    // public static WorkingCopyViewModel Create(string workingCopyPath)
    // {
    //     var messenger = new WeakReferenceMessenger();
    //     return new WorkingCopyViewModel
    //     {
    //         Messenger = messenger,
    //         WorkingCopyPath = workingCopyPath,
    //     }.RegisterMessages();
    // }


    // private WorkingCopyViewModel RegisterMessages()
    // {
    //     this.RegisterAllMessages(Messenger);
    //     return this;
    // }
    //

    public void Receive(OnGetDialogHostId message)
    {
        message.Reply(DialogHostId);
    }

    public void Receive(OnWorkingCopyViewEnabled message)
    {
        IsEnabled = message.Value;
    }

    public void Receive(OnInitializeRepository message)
    {
        var contextNotifierDelegate = new ContextNotifierDelegate
        {
            DialogHostId =  DialogHostId,
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

                await context.InitializeRepository(message.Value, new InitializeRepositoryNotifierDelegate
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
                                Content = $"Checkout from {message.Value.Remote}",
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

            await OverlayDialog.ShowModal<ImportProcessDialog, ImportProcessDialogModel>(importProcessDialogModel, options: options, hostId: DialogHostId);
            if (importProcessDialogModel.Error is null)
            {
                await LocalViewModel.Status();
            }
        });

    }

    public void Receive(OnNotWorkingCopy message)
    {
        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var result = await MessageBox.ShowOverlayAsync(title: "Error", hostId: DialogHostId, message: "Not a working copy, initialize now", button: MessageBoxButton.YesNo);
            if (result is MessageBoxResult.No)
            {
                // Manager.MainWindow.Send(new OnRemoveTabByLocalViewModel(this));
                Manager.Default.Send(new OnRemoveTab(Token), Manager.MainWindowToken);
            }
            else
            {
                var dialogOptions = new OverlayDialogOptions
                {
                    Title = "Test",
                    IsCloseButtonVisible = true,
                    Buttons = DialogButton.None
                };
                await OverlayDialog.ShowModal<ImportDialog, ImportDialogModel>(new ImportDialogModel()
                {
                    Path = WorkingCopyPath,
                }, hostId: DialogHostId, options: dialogOptions);
            }

        });
    }
}