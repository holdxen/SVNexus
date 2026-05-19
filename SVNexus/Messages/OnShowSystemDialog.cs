using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Utils;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnShowSystemDialog: AsyncRequestMessage<Unit>
{
    public required SystemDialogModel Dialog { get; set; }
}