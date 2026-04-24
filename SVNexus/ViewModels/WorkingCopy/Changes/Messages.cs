using System.Collections.Generic;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public static class Messages
{
    public class OnSelectedItemChanged(StatusEntry? value) : ValueChangedMessage<StatusEntry?>(value);
    
    public class OnSelectedItemsChanged(List<StatusEntry> value) : ValueChangedMessage<List<StatusEntry>>(value);
}