using System;
using System.Collections.Generic;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnGetTabByWorkspaceRoot: RequestMessage<Guid?>
{
    public required string Root { get; init; }
}


public class OnGetCurrentTab: RequestMessage<Guid>;