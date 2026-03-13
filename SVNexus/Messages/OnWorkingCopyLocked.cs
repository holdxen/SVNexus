using System.Collections.Generic;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Generated;

namespace SVNexus.Messages;

public class OnWorkingCopyLocked(List<StatusEntry> value) : ValueChangedMessage<List<StatusEntry>>(value);