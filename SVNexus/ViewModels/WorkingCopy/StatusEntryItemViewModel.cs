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
    [NotifyPropertyChangedFor(nameof(Path))]
    [NotifyPropertyChangedFor(nameof(IsDelete))]
    [NotifyPropertyChangedFor(nameof(RelativePath))]
    [NotifyPropertyChangedFor(nameof(RelativeDirectory))]
    [NotifyPropertyChangedFor(nameof(IsLocked))]
    public required partial StatusEntry Entry { get; set; }

    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(RelativePath))]
    [NotifyPropertyChangedFor(nameof(RelativeDirectory))]
    public required partial string RelateTo { get; set; }


    public bool IsLocked => Entry.Lock is not null;
    
    public string FileName => Entry.Path == RelateTo ? "/" : Entry.Path.GetFileName();

    public string StatusIcon => Entry.NodeStatus.Icon();

    public string StatusToolTip => Entry.NodeStatus.ToString();
    
    public string KindIcon => Entry.NodeKind.Icon();

    public string Path => Entry.Path;
    
    public string RelativePath => Entry.Path == RelateTo ? "/" : Entry.Path.TrimStartString(RelateTo).TrimStartPathSeparatorChar();

    public string RelativeDirectory
    {
        get
        {
            if (string.IsNullOrEmpty(RelateTo))
            {
                return Entry.Path;
            }
            if (RelateTo == Entry.Path)
            {
                return string.Empty;
            }
            var contain = Entry.Path.TrimStartString(RelateTo)
                .TrimStartPathSeparatorChar().GetDirectoryName();
            // return string.IsNullOrEmpty(contain) ? "/" : contain;
            return contain ?? string.Empty;
        }
    }
    
    public bool IsDelete => Entry.NodeStatus == WorkingCopyStatus.Deleted;
    
    
}