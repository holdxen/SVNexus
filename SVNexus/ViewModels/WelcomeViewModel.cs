using System;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.Views;
using SVNexus.Views.WorkingCopy;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class WelcomeViewModel: ViewModelBase, IRecipient<OnCheckout>, IRecipient<OnExport>
{
    public override bool KeepAlive { get; set; } = true;
    
    
    // private WeakReferenceMessenger Messenger { get; } = new();

    
    [ObservableProperty]
    private string? _dialogHostId;

    [RelayCommand]
    private async Task ShowCheckoutDialog()
    {
        var options = new OverlayDialogOptions
        {
            Title = "Checkout",
            IsCloseButtonVisible = true,
            Buttons = DialogButton.None
        };
        
        Console.WriteLine($"Show dialog {DialogHostId}");
        await OverlayDialog.ShowModal<Views.CheckoutOrExportDialog, CheckoutOrExportDialogModel>(new CheckoutOrExportDialogModel(), DialogHostId, options: options);
        Console.WriteLine("Close dialog");
    }


    [RelayCommand]
    private async Task OpenLocalRepository()
    {
        var options = new FolderPickerOpenOptions()
        {
            AllowMultiple = false,
            Title = "Select a local repository",
        };
        
        
        var result = await Manager.Default.Send(new OnFolderPickerOpen(options), Manager.MainWindowToken);
        if (result.Count == 1)
        {

            var workingCopyView = new WorkingCopyViewModel
            {
                WorkingCopyPath = result[0].Path.AbsolutePath,
            };
            Console.WriteLine("Send Add tab: {0} {1}", workingCopyView.WorkingCopyPath, Manager.MainWindowToken);
            Manager.Default.Send(new OnAddTab(new MainWindowViewModel.TabItemViewViewModel()
            {
                Closable = true,
                Content = workingCopyView,
                Text = result[0].Name,
            }), Manager.MainWindowToken);
        }
    }

    public void Receive(OnCheckout message)
    {
        Console.WriteLine($"OnCheckout {message}");
        var messenger = new WeakReferenceMessenger();
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.Value.Url,
            Path = message.Value.Path,
            Messenger = messenger
        };
        


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStart(message.Value.Path).ToString();
                    model.ProcessLogItems.Add(new CheckoutOrExportProcessDialogModel.ProcessLogItemViewModel()
                    {
                        Action = notify.Action.ToString(),
                        MimeType =  notify.MimeType ?? "",
                        Path = path
                    });
                    model.CurrentFile = path;
                });
            },
            ProgressNotifyAction = (downloaded, total) =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    model.Total = total;
                    model.Downloaded = downloaded;
                });
            },
            DialogHostId = DialogHostId
        };
        
        
        var createContextOptions = Engine.Engine.Instance.MakeCreateContextOptions(contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        messenger.Register<OnCancel>(contextNotifier, (recipient, cancel) =>
        {
            (recipient as ContextNotifierDelegate)!.CancelMessage = "User cancel";
        });

        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Checkout",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };
            Console.WriteLine("Show dialog");
            await OverlayDialog.ShowModal<CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: DialogHostId, options: options);
        });
        
        Task.Run(async () =>
        {
            try
            {
        
                Console.WriteLine("Checkout now");
                await context.Checkout(message.Value);
                Console.WriteLine("Checkout now finished");
                Dispatcher.UIThread.Invoke(() => { model.IsCompleted = true; });
            }
            catch (System.Exception e)
            {
                model.Error = e.HumanReadableMessage;
            }
            finally
            {
                context.Dispose();
            }
        });
        
        
        


    }


    [RelayCommand]
    private async Task ShowExportDialog()
    {
        var exportDialogModel = new ExportDialogModel();
        
        var options = new OverlayDialogOptions
        {
            Title = "Export",
            IsCloseButtonVisible = true,
            Buttons = DialogButton.None
        };
        
        await OverlayDialog.ShowModal<ExportDialog, ExportDialogModel>(exportDialogModel, options: options, hostId: DialogHostId);
    }

    public void Receive(OnExport message)
    {
        
        Console.WriteLine($"OnCheckout {message}");
        var messenger = new WeakReferenceMessenger();
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.Value.FromPathOrUrl,
            Path = message.Value.ToPath,
            Messenger = messenger
        };
        


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStart(message.Value.ToPath).ToString();
                    model.ProcessLogItems.Add(new CheckoutOrExportProcessDialogModel.ProcessLogItemViewModel()
                    {
                        Action = notify.Action.ToString(),
                        MimeType =  notify.MimeType ?? "",
                        Path = path
                    });
                    model.CurrentFile = path;
                });
            },
            ProgressNotifyAction = (downloaded, total) =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    model.Total = total;
                    model.Downloaded = downloaded;
                });
            },
            DialogHostId = DialogHostId
        };
        
        
        var createContextOptions = Engine.Engine.Instance.MakeCreateContextOptions(contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        messenger.Register<OnCancel>(contextNotifier, (recipient, cancel) =>
        {
            (recipient as ContextNotifierDelegate)!.CancelMessage = "User cancel";
        });

        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Export",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };
            Console.WriteLine("Show dialog");
            await OverlayDialog.ShowModal<CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: DialogHostId, options: options);
        });
        
        Task.Run(async () =>
        {
            try
            {
                await context.Export(message.Value);
                Dispatcher.UIThread.Invoke(() => { model.IsCompleted = true; });
            }
            catch (System.Exception e)
            {
                model.Error = e.HumanReadableMessage;
            }
            finally
            {
                context.Dispose();
            }
        });
        
    }
}