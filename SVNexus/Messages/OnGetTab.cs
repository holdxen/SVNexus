using System;
using System.Collections.Generic;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnGetTabByWorkspaceRoot: RequestMessage<MainWindowViewModel.TabItemViewModel?>
{
    public required string Root { get; set; }
}