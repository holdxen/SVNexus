using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Utils;

namespace SVNexus.Messages;

public static class ClipBoardMessages
{
    public class SetText : AsyncRequestMessage<Unit>
    {
        public required string Text { get; set; }
    }
}