using System;
using System.Linq;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class InfoDialogModel(ViewModelBase parent) : DialogModelBase(parent)
{
    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        IsCloseButtonVisible = false,
        Buttons = DialogButton.None,
        Title = "Information",
    };

    [ObservableProperty] public partial string Path { get; set; } = string.Empty;

    [ObservableProperty] public partial Revision PegRevision { get; set; } = new Revision.Base();

    [ObservableProperty] public partial Revision Revision { get; set; } = new Revision.Working();

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(RevisionText))]
    [NotifyPropertyChangedFor(nameof(Url))]
    [NotifyPropertyChangedFor(nameof(Name))]
    [NotifyPropertyChangedFor(nameof(Kind))]
    [NotifyPropertyChangedFor(nameof(WorkingCopyFormat))]
    [NotifyPropertyChangedFor(nameof(RepositoryUuid))]
    [NotifyPropertyChangedFor(nameof(RepositoryRootUrl))]
    [NotifyPropertyChangedFor(nameof(LastChangedAuthor))]
    [NotifyPropertyChangedFor(nameof(LastChangedRevision))]
    [NotifyPropertyChangedFor(nameof(LastChangedDate))]
    [NotifyPropertyChangedFor(nameof(Size))]
    [NotifyPropertyChangedFor(nameof(HumanSize))]
    [NotifyPropertyChangedFor(nameof(Lock))]
    public partial InfoEntry? Entry { get; set; }

    public string? RevisionText => Entry?.Revision?.ToString();
    
    public string? Url => Entry?.Url;

    public string? Name => Entry?.Url.GetFileName();

    public string? Kind => Entry?.Kind.ToString();

    public string? WorkingCopyFormat => Entry?.WorkingCopyInfo?.WorkingCopyFormat.ToString();
    
    public string? RepositoryUuid => Entry?.RepositoryUuid;
    
    public string? RepositoryRootUrl => Entry?.RepositoryRootUrl;
    
    public string? LastChangedAuthor => Entry?.LastChangedAuthor;

    public string? LastChangedRevision => Entry?.LastChangedRevision?.ToString();
    
    public string? LastChangedDate => Entry?.LastChangedDate.Map(d => DateTimeOffset.FromUnixTimeMilliseconds(d / 1000).UtcDateTime.ToLocalTime().ToString("yyyy-MM-dd HH:mm:ss"));
    
    public string? Size => Entry?.Size?.ToString();
    
    public Lock? Lock => Entry?.Lock;
    
    public string? LockCreationDate => Entry?.Lock?.CreationDate.Map(d => DateTimeOffset.FromUnixTimeMilliseconds(d / 1000).UtcDateTime.ToLocalTime().ToString("yyyy-MM-dd HH:mm:ss"));
    
    public string? LockExpirationDate => Entry?.Lock?.ExpirationDate.Map(d => DateTimeOffset.FromUnixTimeMilliseconds(d / 1000).UtcDateTime.ToLocalTime().ToString("yyyy-MM-dd HH:mm:ss"));
    


    public string? HumanSize => Entry?.Size?.Map(e =>
    {
        var opt = new FormatSizeOptions(e);
        return opt.Format();
    });



    [RelayCommand]
    private async Task OnLoaded()
    {
        if (Entry is not null)
        {
            return;
        }
        var context = SendMessage(new OnGetContext()).Response;

        var infoOptions = new InfoOptions(Path, PegRevision, Revision, Depth.Empty, true, true, false);
        
        var result = await context.Info(infoOptions);

        if (result.Entries.Count == 0)
        {
            Logger.Warn($"Failed to show information of: {Path}");
            return;
        }


        Entry = result.Entries.Values.First();

    }
    
    
    
    protected override Task OnConfirm()
    {
        return Task.CompletedTask;
    }
}