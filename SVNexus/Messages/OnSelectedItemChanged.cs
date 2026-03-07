using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnSelectedItemChanged(string? value) : ValueChangedMessage<string?>(value);