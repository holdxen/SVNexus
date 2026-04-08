using System;
using Avalonia;
using Avalonia.Controls;
using SVNexus.Generated;

namespace SVNexus.Extension;

public static class Extension
{

    public static string LogChangedPathActionIcon(this LogChangedPathAction action)
    {
        var icon = action switch
        {
            LogChangedPathAction.Add => "Icons.Status-Added",
            LogChangedPathAction.Delete => "Icons.Status-Deleted",
            LogChangedPathAction.Replace => "Icons.Status-Replaced",
            LogChangedPathAction.Modify => "Icons.Status-Modified",
            _ => throw new ArgumentOutOfRangeException(nameof(action), action, null)
        };

        var pathIcon = (Application.Current!.FindResource(icon) as string)!;
            

        return pathIcon;
    }

    public static string NodeStatusIcon(this WorkingCopyStatus status)
    {
        var icon = status switch
        {
            WorkingCopyStatus.None => "Icons.Status-Normal",
            WorkingCopyStatus.Unversioned => "Icons.Status-Unversioned",
            WorkingCopyStatus.Normal => "Icons.Status-Normal",
            WorkingCopyStatus.Added => "Icons.Status-Added",
            WorkingCopyStatus.Missing => "Icons.Status-Missing",
            WorkingCopyStatus.Deleted => "Icons.Status-Deleted",
            WorkingCopyStatus.Replaced => "Icons.Status-Replaced",
            WorkingCopyStatus.Modified => "Icons.Status-Modified",
            WorkingCopyStatus.Merged => "Icons.Status-Merged",
            WorkingCopyStatus.Conflicted => "Icons.Status-Conflicted",
            WorkingCopyStatus.Ignored => "Icons.Status-Ignored",
            WorkingCopyStatus.Obstructed => "Icons.Status-Obstructed",
            WorkingCopyStatus.External => "Icons.Status-External",
            WorkingCopyStatus.Incomplete => "Icons.Status-Incomplete",
            _ => throw new ArgumentOutOfRangeException(nameof(status), status, null)
        };

        var pathIcon = (Application.Current!.FindResource(icon) as string)!;
            

        return pathIcon;
    }
    
    public static string NodeKindIcon(this NodeKind kind)
    {
        var key = kind switch
        {
            NodeKind.File => "Icons.File-Normal",
            NodeKind.Directory => "Icons.Directory-Close",
            NodeKind.Unknown => "Icons.File-Unknown",
            NodeKind.Symlink => "Icons.File-Symlink", 
            _ => "Icons.FileUnknown"
        };

        return (Application.Current!.FindResource(key) as string)!;
    }
}