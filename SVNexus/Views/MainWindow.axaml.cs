using System;
using Avalonia.Controls;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Inject;
using SVNexus.Messages;
using Ursa.Controls;

namespace SVNexus.Views;

public partial class MainWindow : Window, IRecipient<OnFolderPickerOpen>, IRecipient<OnNotification>
{
    
    private readonly WindowNotificationManager _notificationManager;
    
    public MainWindow()
    {
        InitializeComponent();
        
        Ambient.SetGuid(this, Manager.MainWindowToken);
        Manager.Default.RegisterAllMessages(this, Manager.MainWindowToken);
        
        _notificationManager = new WindowNotificationManager(this);
    }
    
    

    public void Receive(OnFolderPickerOpen message)
    {
        var top = GetTopLevel(this);
        if (top == null)
        {
            return;
        }

        
        message.Reply(top.StorageProvider.OpenFolderPickerAsync(message.Options));
        

    }

    public void Receive(OnNotification message)
    {
        _notificationManager.Show(message.Value);
    }
}