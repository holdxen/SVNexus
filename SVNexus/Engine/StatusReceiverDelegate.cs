using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class StatusReceiverDelegate: StatusReceiver
{
    public Action<StatusEntry>? OnStatusEntryAction { get; init; }
    public void OnStatusEntry(StatusEntry entry)
    {
        OnStatusEntryAction?.Invoke(entry);
    }
}