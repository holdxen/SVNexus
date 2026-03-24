using System;
using System.Collections.Generic;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Extension;
using SVNexus.Generated;

using SystemPath = System.IO.Path;

namespace SVNexus.ViewModels.WorkingCopy.History;

public partial class HistoryDetailViewModel: ViewModelBase
{
    [ObservableProperty]
    public required partial LogEntry Entry { get; set; }
    
    
    public string? Author => Entry.Author;
    
    public string? Message => Entry.Message;
    
    public uint? Revision => Entry.Revision;
    
    public string DateTimeText => DateTimeOffset.FromUnixTimeMilliseconds(Entry.Date.GetValueOrDefault() / 1000).UtcDateTime.ToString("u");
    
    public required string RelateToRoot { get; set; }
    
    public partial class ChangeItemViewModel: ViewModelLite
    {
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(Icon))]
        public required partial LogChangedPathEntry Entry { get; set; }
        
        public required string RelativeToRoot { get; set; }

        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(RelativePath))]
        public partial string Path { get; set; } = string.Empty;

        public string Icon => Entry.Action.LogChangedPathActionIcon();

        public string RelativePath => Path == RelativeToRoot
            ? "/"
            : Path.TrimStartString(RelativeToRoot).TrimStartPathSeparatorChar();

    }


    public List<ChangeItemViewModel> ChangeItems =>
        Entry.ChangedPathEntries.Select(p => new ChangeItemViewModel { RelativeToRoot = RelateToRoot, Entry = p.Value, Path = p.Key }).ToList();


}