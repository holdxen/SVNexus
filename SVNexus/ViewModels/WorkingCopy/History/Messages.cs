using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.History;

public static class Messages
{
    public class OnGetWorkingCopyUrl: RequestMessage<string>;
}