using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnSelectedItemChanged(object? value) : ValueChangedMessage<object?>(value);