using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy;

// public interface ITargetItemViewModel
// {
//     public string FileName { get; }
//     
//     public string RelativePath { get; }
//     
//     public string RelativeDirectory { get; }
//     
//     public string KindIcon { get; }
//     
//     public string StatusIcon { get; }
//     
//     public string StatusToolTip { get; }
//     
//     public string Path { get; }
//     
// }


public partial class TargetItemViewModel(ViewModelBase? parent = null): ViewModelBase(parent)
{
    [ObservableProperty] public partial string KindIcon { get; set; } = string.Empty;

    [ObservableProperty] public partial string TextToolTip { get; set; } = string.Empty;
    
    [ObservableProperty] public partial bool IsDelete { get; set; }

    [ObservableProperty] public partial string FileName { get; set; } = string.Empty;

    [ObservableProperty] public partial string RelativeDirectory { get; set; } = string.Empty;

    [ObservableProperty] public partial bool ShowRelateDirectory { get; set; }
    
    [ObservableProperty] public partial bool IsLocked { get; set; }
    
    [ObservableProperty] public partial string StatusToolTip { get; set; } = string.Empty;
    
    [ObservableProperty] public partial bool IsLoading { get; set; }

    [ObservableProperty] public partial string StatusIcon { get; set; } = string.Empty;
    
    public required string Path { get; set; }

    public static TargetItemViewModel From(StatusEntry statusEntry, bool absolute = false, string? relateTo = null)
    {
        string fileName;
        if (absolute)
        {
            fileName = statusEntry.Path;
        }
        else
        {
            fileName = statusEntry.Path == relateTo ? "/" : statusEntry.Path.GetFileName();
        }
        
        string relativeDirectory;

        if (absolute)
        {
            relativeDirectory = string.Empty;
        }
        else
        {
            if (string.IsNullOrEmpty(relateTo))
            {
                relativeDirectory = statusEntry.Path.GetDirectoryName() ?? string.Empty;
            }
            else if (statusEntry.Path == relateTo)
            {
                relativeDirectory = "/";
            }
            else
            {
                relativeDirectory = statusEntry.Path.TrimStartString(relateTo).TrimStartPathSeparatorChar().GetDirectoryName() ?? string.Empty;
            }
        }

        return new TargetItemViewModel()
        {
            Path = statusEntry.Path,
            KindIcon =  statusEntry.NodeKind.Icon(),
            TextToolTip = statusEntry.Path,
            IsDelete = statusEntry.NodeStatus == WorkingCopyStatus.Deleted,
            FileName = fileName,
            RelativeDirectory = relativeDirectory,
            ShowRelateDirectory = true,
            IsLocked = statusEntry.Lock is not null,
            StatusToolTip = statusEntry.NodeStatus.ToString(),
            StatusIcon = statusEntry.NodeStatus.Icon(),
            IsLoading = false,
        };
    }
}