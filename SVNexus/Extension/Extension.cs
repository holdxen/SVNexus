using System;
using Avalonia;
using Avalonia.Controls;
using SVNexus.Generated;

namespace SVNexus.Extension;

public static class Extension
{
    public static string TrimStartString(this string self, string s, StringComparison comparisonType = StringComparison.Ordinal)
    {
        return self.StartsWith(s) ? self[s.Length..] : self;
    }

    public static string NodeStatusIcon(this NodeStatus status)
    {
        var icon = status switch
        {
            NodeStatus.None => "Icons.Status-Normal",
            NodeStatus.Unversioned => "Icons.Status-Unversioned",
            NodeStatus.Normal => "Icons.Status-Normal",
            NodeStatus.Added => "Icons.Status-Added",
            NodeStatus.Missing => "Icons.Status-Missing",
            NodeStatus.Deleted => "Icons.Status-Deleted",
            NodeStatus.Replaced => "Icons.Status-Replaced",
            NodeStatus.Modified => "Icons.Status-Modified",
            NodeStatus.Merged => "Icons.Status-Merged",
            NodeStatus.Conflicted => "Icons.Status-Conflicted",
            NodeStatus.Ignored => "Icons.Status-Ignored",
            NodeStatus.Obstructed => "Icons.Status-Obstructed",
            NodeStatus.External => "Icons.Status-External",
            NodeStatus.Incomplete => "Icons.Status-Incomplete",
            _ => throw new ArgumentOutOfRangeException()
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
            NodeKind.Unknown => "Icons.FileUnknown",
            NodeKind.Symlink => "Icons.File-Symlink", 
            _ => "Icons.FileUnknown"
        };

        return (Application.Current!.FindResource(key) as string)!;
    }
}