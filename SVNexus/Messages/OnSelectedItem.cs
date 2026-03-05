using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnSelectedItem(string? value) : ValueChangedMessage<string?>(value);