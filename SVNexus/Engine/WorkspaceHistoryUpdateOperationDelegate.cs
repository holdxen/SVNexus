using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class WorkspaceHistoryUpdateOperationDelegate: WorkspaceHistoryUpdateOperation
{
    public required Func<WorkspaceHistory, WorkspaceHistory> UpdateFunc { get; init; }
    public WorkspaceHistory Update(WorkspaceHistory v)
    {
        return UpdateFunc.Invoke(v);
    }
}