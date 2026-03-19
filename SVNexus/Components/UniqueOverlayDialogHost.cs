using System;
using Avalonia.Interactivity;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Inject;
using SVNexus.Messages;
using Ursa.Controls;

namespace SVNexus.Components;

public class UniqueOverlayDialogHost: OverlayDialogHost, IRecipient<OnGetDialogHostId>
{
    
    private readonly string _hostId = Guid.NewGuid().ToString();

    public UniqueOverlayDialogHost()
    {
        HostId = _hostId;
    }
    
    public void Receive(OnGetDialogHostId message)
    {
        message.Reply(_hostId);
    }

    protected override void OnLoaded(RoutedEventArgs e)
    {
        base.OnLoaded(e);

        var token = Ambient.GetGuid(this);
        
        Manager.Default.RegisterAllMessages(this, token);
    }

    protected override Type StyleKeyOverride { get; } = typeof(OverlayDialogHost);
}