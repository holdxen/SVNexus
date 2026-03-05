using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class LogReceiverDelegate: LogReceiver
{
    
    public Action<LogEntry>? OnLogEntryAction { get; init; }
    
    public void OnLogEntry(LogEntry logEntry)
    {
        OnLogEntryAction?.Invoke(logEntry);
    }
}