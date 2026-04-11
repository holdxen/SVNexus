using System.Diagnostics;
using SVNexus.Generated;

namespace SVNexus.Extension;

public static class WorkspaceHistoryExtension
{
    extension(WorkspaceHistory history)
    {
        public WorkspaceHistory WithIsStar(bool isStar)
        {
            return history switch
            {
                WorkspaceHistory.Repository repository => repository with { Star = isStar },
                WorkspaceHistory.WorkingCopy workingCopy => workingCopy with { Star = isStar },
                _ => throw new UnreachableException()
            };
        }

        public string Uuid
        {
            get
            {
                return history switch
                {
                    WorkspaceHistory.Repository repository => repository.Uuid,
                    WorkspaceHistory.WorkingCopy workingCopy => workingCopy.Uuid,
                    _ => throw new UnreachableException()
                };
            }
        }
    }
}