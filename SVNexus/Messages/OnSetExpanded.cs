using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnSetExpanded(bool value) : ValueChangedMessage<bool>(value);