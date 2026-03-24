using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnSetChecked(bool value) : ValueChangedMessage<bool>(value);