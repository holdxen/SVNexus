using System;
using System.Text.Json;
using System.Threading.Tasks;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.Views;
using SVNexus.Views.WorkingCopy;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class WelcomeViewModel: ViewModelBase, IRecipient<OnCheckout>, IRecipient<CheckoutOrExportProcessDialogModel.OnCancel>
{
    public override bool KeepAlive { get; set; } = true;
    
    
    private WeakReferenceMessenger Messenger { get; } = new();


    public WelcomeViewModel()
    {
        Messenger.Register<OnCheckout>(this);
        Messenger.Register<CheckoutOrExportProcessDialogModel.OnCancel>(this);
    }
    
    
    [ObservableProperty]
    private string? _dialogHostId;

    [RelayCommand]
    private async Task ShowCheckoutDialog()
    {
        var options = new OverlayDialogOptions
        {
            Title = "Test",
            IsCloseButtonVisible = true,
            Buttons = DialogButton.None
        };
        
        Console.WriteLine($"Show dialog {DialogHostId}");
        await OverlayDialog.ShowModal<Views.CheckoutOrExportDialog, CheckoutOrExportDialogModel>(new CheckoutOrExportDialogModel()
        {
            Messenger = Messenger
        }, DialogHostId, options: options);
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
        
        
        var result = await WeakReferenceMessenger.Default.Send(new OnFolderPickerOpen(options));
        if (result.Count == 1)
        {

            var workingCopyView = WorkingCopyViewModel.Create(result[0].Path.AbsolutePath);

            WeakReferenceMessenger.Default.Send(new OnAddTab(new MainWindowViewModel.TabItemViewViewModel()
            {
                Closable = true,
                Content = workingCopyView,
                Text = result[0].Name,
            }));
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
                    model.ProcessDetailItems.Add(new CheckoutOrExportProcessDialogModel.ProcessDetailItemViewModel()
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
            SslServerTrustPromptFunc = (string realm, uint failures, SslServerCertInfo info, bool maySave) =>
            {
                return Dispatcher.UIThread.InvokeAsync(async () =>
                {
                    var options = new OverlayDialogOptions()
                    {
                        IsCloseButtonVisible = false,
                        Title = "Info",
                        Buttons = DialogButton.None
                    };
                    var trustServerDialogModel = new TrustServerDialogModel()
                    {
                        Realm = realm,
                        Issuer = info.Issuer,
                        AsciiCert = info.AsciiCert,
                        Hostname = info.Hostname,
                        ValidFrom = info.ValidFrom,
                        ValidUntil = info.ValidUntil,
                        Fingerprint = info.Fingerprint,
                        Savable = maySave,
                    };
                    await OverlayDialog.ShowModal<TrustServerDialog, TrustServerDialogModel>(trustServerDialogModel, options: options, hostId: DialogHostId);
                    return new TrustServer(failures, trustServerDialogModel.Save);
                }).Result;
            }
        };
        
        
        var createContextOptions = Engine.Engine.Instance.MakeCreateContextOptions(contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        messenger.Register<CheckoutOrExportProcessDialogModel.OnCancel>(contextNotifier, (recipient, cancel) =>
        {
            (recipient as ContextNotifierDelegate)!.CancelMessage = "User cancel";
        });

        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Checkout",
                IsCloseButtonVisible = true,
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
                model.Error = e.Message;
                if (e is Generated.Exception.SvnException svnException)
                {
                    var options = new JsonSerializerOptions
                    {
                        PropertyNameCaseInsensitive = true,
                        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
                    };
                    var error = JsonSerializer.Deserialize<SvnError>(svnException.Message, options)!;
                    var errorNumber = new SvnErrnoConstants();
                    if (errorNumber.IsWcNotDirectory(error.Code))
                    {
                        Console.WriteLine($"WcNotDirectory: {error.Code}");
                    }
                }

                Console.WriteLine("Got exception: {0}", e);
            }
            finally
            {
                context.Dispose();
            }
        });
        
        
        


    }

    public void Receive(CheckoutOrExportProcessDialogModel.OnCancel message)
    {
        var model = message.Model;
        model.Shutdown();
    }
}