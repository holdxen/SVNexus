using System;
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
using SVNexus.Views.WorkingCopy;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class WelcomeViewModel: ViewModelBase, IRecipient<OnCheckout>
{
    public override bool KeepAlive { get; set; } = true;
    
    
    private readonly WeakReferenceMessenger _messenger = new();


    public WelcomeViewModel()
    {
        _messenger.Register(this);
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
            Messenger = _messenger
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
            // var localModel = new LocalViewModel
            // {
            //     WorkingCopyPath = result[0].Path.AbsolutePath
            // };
            // localModel.Initialize();

            // var workingCopyView = new WorkingCopyViewModel()
            // {
            //     WorkingCopyPath = result[0].Path.AbsolutePath
            // };
            //
            // workingCopyView.Initialize();

            var workingCopyView = WorkingCopyViewModel.Create(result[0].Path.AbsolutePath);

            WeakReferenceMessenger.Default.Send(new OnAddTab(new MainWindowViewModel.TabItemViewViewModel()
            {
                Closable = true,
                Content = workingCopyView,
                Text = result[0].Name
            }));
        }
    }

    public void Receive(OnCheckout message)
    {
        Console.WriteLine($"OnCheckout {message}");
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.Value.Url,
            Path = message.Value.Path
        };


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = (notify) =>
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
            }
        };
        
        
        var createContextOptions = Engine.Engine.Instance.MakeCreateContextOptions(contextNotifier);

        var context = AsyncContext.Create(createContextOptions);

        Dispatcher.UIThread.InvokeAsync((Func<Task>)(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Checkout",
                IsCloseButtonVisible = true,
                Buttons = DialogButton.None
            };
            Console.WriteLine("Show dialog");
            await OverlayDialog.ShowModal<Views.CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: DialogHostId, options: options);
        }));
        
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
                Console.WriteLine(e);
                throw;
            }
            finally
            {
                context.Dispose();
            }
        });
        
        
        


    }

}