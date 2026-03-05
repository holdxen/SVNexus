using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

public class OnNotWorkingCopy(string value) : ValueChangedMessage<string>(value);