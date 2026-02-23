using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnWorkingCopyViewEnabled(bool value) : ValueChangedMessage<bool>(value);