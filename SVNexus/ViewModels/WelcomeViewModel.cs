using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class WelcomeViewModel(ViewModelBase parent): ViewModelBase(parent)//, IRecipient<OnCheckout>, IRecipient<OnExport>
{
    [RelayCommand]
    private async Task ShowCheckoutDialog()
    {
        var options = new OverlayDialogOptions
        {
            Title = "Checkout",
            IsCloseButtonVisible = true,
            Buttons = DialogButton.None
        };

        var hostId = SendMessage(new OnGetDialogHostId());

        var model = new CheckoutOrExportDialogModel();
        
        await OverlayDialog.ShowModal<CheckoutOrExportDialog, CheckoutOrExportDialogModel>(model, hostId, options: options);
        if (model.Options is not null)
        {
            Receive(model.Options);
        }
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
            
            var path = result[0].Path.AbsolutePath.TrimEndPathSeparatorChar();


            var tab = new MainWindowViewModel.TabItemViewModel(GetParent<MainWindowViewModel>()!)
            {
                Closable = true,
                Text = result[0].Name
            };
            
            var content = new WorkspaceViewModel(path, tab);
            tab.Content = content;

            SendMessage(new OnAddTab(tab));
        }
    }

    public void Receive(CheckoutOptions message)
    {
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId());
        var messenger = new WeakReferenceMessenger();
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.Url,
            Path = message.Path,
            Messenger = messenger
        };
        


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStart(message.Path).ToString();
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
            DialogHostId = hostId
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
            await OverlayDialog.ShowModal<CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: hostId, options: options);
        });
        
        Task.Run(async () =>
        {
            try
            {
        
                Console.WriteLine("Checkout now");
                await context.Checkout(message);
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
        
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<ExportDialog, ExportDialogModel>(exportDialogModel, options: options, hostId: hostId);
        if (exportDialogModel.Options is not null)
        {
            Receive(exportDialogModel.Options);
        }
        
    }

    public void Receive(ExportOptions message)
    {
        
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId());
        var messenger = new WeakReferenceMessenger();
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.FromPathOrUrl,
            Path = message.ToPath,
            Messenger = messenger
        };
        


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStart(message.ToPath).ToString();
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
            DialogHostId = hostId
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
            await OverlayDialog.ShowModal<CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: hostId, options: options);
        });
        
        Task.Run(async () =>
        {
            try
            {
                await context.Export(message);
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