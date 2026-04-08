using System;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Generated;

namespace SVNexus.Messages;

public class OnSetWorkspaceHistory(Func<WorkspaceHistory, WorkspaceHistory> value)
    : ValueChangedMessage<Func<WorkspaceHistory, WorkspaceHistory>>(value);