using System;

namespace SVNexus.Utils;

public class DropGuard(Action action): IDisposable
{
    public void Dispose()
    {
        action();
        GC.SuppressFinalize(this);
    }
}