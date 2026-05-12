using System;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnSwitchTab: RequestMessage<bool>
{
    public required Guid Tab { get; init; }
}