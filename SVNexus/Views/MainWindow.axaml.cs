using System;
using System.Threading.Tasks;
using Avalonia.Controls;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Utils;
using Ursa.Controls;
using WindowNotificationManager = Ursa.Controls.WindowNotificationManager;

namespace SVNexus.Views;

public partial class MainWindow : Window, 
    IRecipient<OnFolderPickerOpen>, 
    IRecipient<OnNotification>, 
    IRecipient<OnShowToast>, 
    IRecipient<OnFilePickerOpen>,
    IRecipient<OnFilePickerSave>,
    IRecipient<ClipBoardMessages.SetText>
{
    
    private readonly WindowNotificationManager _notificationManager;
    private readonly WindowToastManager _toastManager;
    
    public MainWindow()
    {
        InitializeComponent();
        
        Manager.Default.RegisterAllMessages(this, Manager.MainWindowToken);
        
        _notificationManager = new WindowNotificationManager(this);
        _toastManager = new WindowToastManager(this);
        
        CatchGlobalExceptions();
    }


    private void CatchGlobalExceptions()
    {
        Dispatcher.UIThread.UnhandledException += (sender, args) =>
        {
            Receive(new OnShowToast()
            {
                Content = $"Unhandled Exception: {args.Exception.Message}",
                Type = NotificationType.Error
            });
        };

        TaskScheduler.UnobservedTaskException += (sender, args) =>
        {
            Receive(new OnShowToast()
            {
                Content =  $"Unobserved Exception: {args.Exception.Message}",
                Type = NotificationType.Error
            });
        };

        AppDomain.CurrentDomain.UnhandledException += (sender, args) =>
        {
            Receive(new OnShowToast()
            {
                Content = $"App domain unhandled Exception: {args.ExceptionObject}",
                Type = NotificationType.Error
            });
        };
    }
    

    public void Receive(OnFolderPickerOpen message)
    {
        var top = GetTopLevel(this);
        if (top == null)
        {
            return;
        }

        
        message.Reply(Dispatcher.UIThread.InvokeAsync(async () => await top.StorageProvider.OpenFolderPickerAsync(message.Options)));
        

    }

    public void Receive(OnNotification message)
    {
        _notificationManager.Show(message);
    }

    public void Receive(OnShowToast message)
    {
        _toastManager.Show(
            message.Content, 
            message.Type, 
            message.Expiration, 
            message.ShowIcon, 
            message.ShowClose, 
            message.OnClick, 
            message.OnClose, 
            message.Classes);
    }

    public void Receive(OnFilePickerOpen message)
    {
        var top = GetTopLevel(this);
        if (top == null)
        {
            return;
        }
        
        message.Reply(top.StorageProvider.OpenFilePickerAsync(message.Options));
    }

    public void Receive(OnFilePickerSave message)
    {
        var top = GetTopLevel(this);
        if (top == null)
        {
            return;
        }
        
        message.Reply(top.StorageProvider.SaveFilePickerAsync(message.Options));
    }

    public void Receive(ClipBoardMessages.SetText message)
    {
        var top = GetTopLevel(this);
        if (top == null)
        {
            message.Reply(Task.FromResult(Unit.Value));
        }
        else
        {
            async Task<Unit> SetText()
            {
                if (top.Clipboard != null)
                {
                    await top.Clipboard.SetTextAsync(message.Text);
                }

                return Unit.Value;
            }

            message.Reply(SetText());
        }
    }
}