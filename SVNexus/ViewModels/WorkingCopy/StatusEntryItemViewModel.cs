using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class StatusEntryItemViewModel(ViewModelBase? parent = null): ViewModelBase(parent)
{
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(FileName))]
    [NotifyPropertyChangedFor(nameof(StatusIcon))]
    [NotifyPropertyChangedFor(nameof(StatusToolTip))]
    [NotifyPropertyChangedFor(nameof(KindIcon))]
    [NotifyPropertyChangedFor(nameof(AbsolutePath))]
    [NotifyPropertyChangedFor(nameof(IsDelete))]
    [NotifyPropertyChangedFor(nameof(RelativePath))]
    [NotifyPropertyChangedFor(nameof(RelativeDirectory))]
    public required partial StatusEntry Entry { get; set; }

    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(RelativePath))]
    [NotifyPropertyChangedFor(nameof(RelativeDirectory))]
    public required partial string RelateTo { get; set; }

    
    public string FileName => Entry.Path.GetFileName();

    public string StatusIcon => Entry.NodeStatus.Icon();
    
    public string StatusToolTip => Entry.NodeStatus.ToString();
    
    public string KindIcon => Entry.NodeKind.Icon();

    public string AbsolutePath => Entry.Path;
    
    public string RelativePath => Entry.Path == RelateTo ? "/" : Entry.Path.TrimStartString(RelateTo).TrimStartPathSeparatorChar();

    public string? RelativeDirectory
    {
        get
        {
            if (RelateTo == Entry.Path)
            {
                return null;
            }
            var contain = Entry.Path.TrimStartString(RelateTo)
                .TrimStartPathSeparatorChar().GetDirectoryName();
            return string.IsNullOrEmpty(contain) ? "/" : contain;
        }
    }
    
    public bool IsDelete => Entry.NodeStatus == WorkingCopyStatus.Deleted;
    
    
}