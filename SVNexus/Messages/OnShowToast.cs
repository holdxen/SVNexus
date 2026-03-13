using System;
using Avalonia.Controls.Notifications;

namespace SVNexus.Messages;

public class OnShowToast
{
    public required object Content { get; set; }
    public NotificationType Type { get; set; }
    public TimeSpan? Expiration { get; set; }
    public bool ShowIcon { get; set; } = true;
    public bool ShowClose { get; set; } = true;
    public Action? OnClick { get; set; }
    public Action? OnClose { get; set; }
    public string[]? Classes { get; set; }
}