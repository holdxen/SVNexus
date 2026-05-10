using System.ComponentModel.DataAnnotations;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Primitives;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class InitializeRepositoryDialogModel (ViewModelBase parent): DialogModelBase(parent)
{
    protected override async Task OnConfirm()
    {
        if (!ValidateAllProperty(out _))
        {
            return;
        }
        string[]? filters = null;

        if (!string.IsNullOrWhiteSpace(Ignore))
        {
            filters = Ignore.Split('\n');
        }
        

        var options = new InitializeRepositoryOptions(Local, Remote, Backup && !string.IsNullOrEmpty(BackupIn) ? BackupIn : null, CommitMessage, IgnoreUnknownNodeTypes, NoIgnore, NoAutoProperties, filters);

        var model = new InitializeRepositoryProcessDialogModel(this)
        {
            InitializeRepositoryOptions = options
        };
        
        await OverlayDialog.ShowStandardAsync<InitializeRepositoryProcessDialog, InitializeRepositoryProcessDialogModel>(model, SendMessage(new OnGetDialogHostId()), OverlayDialogOptions);

        
        Ok();
    }

    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Import and checkout",
        IsCloseButtonVisible = false,
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        VerticalScrollBarVisibility = ScrollBarVisibility.Disabled,
        Buttons = DialogButton.None,
    };


    [Required]
    public string Local
    {
        get;
        set { SetProperty(ref field, value); }
    } = string.Empty;

    [Required]
    public string Remote { get; set => SetProperty(ref field, value); } = string.Empty;


    [ObservableProperty] public partial string BackupIn { get; set; } = string.Empty;

    [Required]
    public string CommitMessage
    {
        get;
        set => SetProperty(ref field, value);
    } = string.Empty;

    [ObservableProperty] public partial bool IgnoreUnknownNodeTypes { get; set; }
    
    [ObservableProperty] public partial bool NoIgnore { get; set; }
    
    [ObservableProperty] public partial bool NoAutoProperties { get; set; }


    [ObservableProperty] public partial bool Backup { get; set; } = true;

    [ObservableProperty] public partial string Ignore { get; set; } = ".git\n.svn\n";
    
    [ObservableProperty] public partial bool IgnoreInBackup { get; set; }


    private SingleTaskQueue? _detectIgnoreFile;

    private void OnLocalChanged(string value)
    {
        if (_detectIgnoreFile is null)
        {
            return;
        }

        _detectIgnoreFile.Run(async token =>
        {
            await DetectIgnoreFile(value, token);
        });
    }

    private async Task DetectIgnoreFile(string path, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        // if (!File.Exists(path))
        // {
        //     return;
        // }

        var file = $"{path}/.gitignore";
        var info = new FileInfo(file);
        
        if (!info.Exists || info.Length > 1024 * 1024 * 32)
        {
            return;
        }
        token.ThrowIfCancellationRequested();
        
        await using var reader = info.OpenRead();
        
        token.ThrowIfCancellationRequested();
        
        await using var memoryStream = new MemoryStream();
        
        token.ThrowIfCancellationRequested();
        
        await reader.CopyToAsync(memoryStream, token);

        token.ThrowIfCancellationRequested();

        var bytes = memoryStream.ToArray();

        var result = await OverlayMessageBox.ShowAsync("Found .gitignore, whether to load it", hostId: SendMessage(new OnGetDialogHostId()), button: MessageBoxButton.YesNo, icon: MessageBoxIcon.Question);
        
        token.ThrowIfCancellationRequested();
        
        if (result == MessageBoxResult.Yes)
        {
           Ignore += "\n" + Encoding.UTF8.GetString(bytes);
        }

    }

    [RelayCommand]
    private async Task SelectedLocalDirectory()
    {
        var options = new FolderPickerOpenOptions()
        {
            AllowMultiple = false,
            Title = "Select a folder to import",
        };
        
        
        var result = await Manager.Default.Send(new OnFolderPickerOpen(options), Manager.MainWindowToken);
        if (result.Count > 0)
        {
            Local = result[0].Path.AbsolutePath;
        }
    }

    [RelayCommand]
    private void OnLoaded()
    {
        _detectIgnoreFile = new SingleTaskQueue();
        if (!string.IsNullOrEmpty(Local))
        {
            _detectIgnoreFile.Run(async token =>
            {
                await DetectIgnoreFile(Local, token);
            });
        }
        BackupIn = Directory.CreateTempSubdirectory().FullName;
    }

    public async Task Show()
    {
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowStandardAsync<InitializeRepositoryDialog, InitializeRepositoryDialogModel>(this, hostId, OverlayDialogOptions);
    }
}