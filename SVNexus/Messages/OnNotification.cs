using CommunityToolkit.Mvvm.Messaging.Messages;
using Ursa.Controls;

namespace SVNexus.Messages;

public class OnNotification(Notification value) : ValueChangedMessage<Notification>(value);