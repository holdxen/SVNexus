using Avalonia.Controls;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Messages;
using Ursa.Controls;

namespace SVNexus.Views;

public partial class MainWindow : Window, IRecipient<OnFolderPickerOpen>, IRecipient<OnNotification>
{
    
    private readonly WindowNotificationManager _notificationManager;
    
    public MainWindow()
    {
        InitializeComponent();
        
        this.RegisterAllMessages(Manager.MainWindow);
        
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