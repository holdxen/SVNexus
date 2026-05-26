using System;
using System.ComponentModel;
using System.ComponentModel.DataAnnotations;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Controls.Primitives;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using Ursa.Controls;
using OperationDepth = SVNexus.Generated.Depth;

namespace SVNexus.ViewModels;

public partial class ImportDialogModel(ViewModelBase parent): DialogModelBase(parent)
{
    
    public enum ValidDepth
    {
        Empty = OperationDepth.Empty,
        Files = OperationDepth.Files,
        Immediates = OperationDepth.Immediates,
        Infinity = OperationDepth.Infinity,
    }
    
    
    public static Type DepthType => typeof(ValidDepth);

    [ObservableProperty]
    public partial ValidDepth Depth { get; set; } = ValidDepth.Infinity;
    
    [Required]
    public string Url { get; set => SetProperty(ref field, value); } = string.Empty;
    
    [Required]
    public string Path { get; set => SetProperty(ref field, value); } = string.Empty;

    
    [Required]
    public string CommitMessage { get; set => SetProperty(ref field, value); } = string.Empty;
    
    
    [ObservableProperty]
    public partial bool NoIgnore { get; set; }
    
    [ObservableProperty]
    public partial bool NoAutoProperties { get; set; }
    
    [ObservableProperty]
    public partial bool IgnoreUnknownNodeTypes { get; set; }

    [ObservableProperty]
    public partial string Filter { get; set; } = ".git\n.svn";

    
    [ObservableProperty]
    public partial bool AppendAble { get; set; }
    


    public override OverlayDialogOptions OverlayDialogOptions { get; } = new()
    {
        Title = "Import",
        Buttons = DialogButton.None,
        HorizontalScrollBarVisibility = ScrollBarVisibility.Disabled,
        VerticalScrollBarVisibility = ScrollBarVisibility.Disabled,
    };
    
    private readonly SingleTaskQueue _detectIgnoreFile = new();

    [RelayCommand]
    private async Task AppendFilter()
    {
        try
        {
            var file = $"{Path}/.gitignore";
            var info = new FileInfo(file);
        
            await using var reader = info.OpenRead();
        
            await using var memoryStream = new MemoryStream();
        
            await reader.CopyToAsync(memoryStream);

            var bytes = memoryStream.ToArray();

            Filter += "\n" + Encoding.UTF8.GetString(bytes);
            
            Manager.Default.Send(new OnShowToast()
            {
                Content = "Append .gitignore file to filter successfully",
                Type = NotificationType.Success
            }, Manager.MainWindowToken);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = "Failed to append .gitignore file to filter: " + e.HumanReadableMessage,
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }
        
    protected override void OnPropertyChanged(PropertyChangedEventArgs e)
    {
        base.OnPropertyChanged(e);
        if (e.PropertyName == nameof(Path))
        {
            OnPathChanged(Path);
        }
    }

    private void OnPathChanged(string value)
    {
        _detectIgnoreFile.Run(token =>
        {
            DetectIgnoreFile(value, token);
        });
    }

    private void DetectIgnoreFile(string path, CancellationToken token)
    {
        token.ThrowIfCancellationRequested();

        var file = $"{path}/.gitignore";
        var info = new FileInfo(file);

        AppendAble = info is { Exists: true, Length: < 1024 * 1024 * 32 };

    }
    
    [RelayCommand]
    private async Task SelectFolder()
    {
        var options = new FolderPickerOpenOptions()
        {
            AllowMultiple = false,
            Title = "Select a folder to import",
        };
        
        
        var result = await Manager.Default.Send(new OnFolderPickerOpen(options), Manager.MainWindowToken);
        if (result.Count > 0)
        {
            Path = result[0].Path.AbsolutePath;
        }
    }

    protected override async Task OnConfirm()
    {
        if (!ValidateAllProperty(out _))
        {
            return;
        }
        
        
        var filters = string.IsNullOrWhiteSpace(Filter) ? null : Filter.Split('\n', StringSplitOptions.RemoveEmptyEntries);

        var options = new ImportOptions(Path, Url, (OperationDepth)Depth, NoIgnore, NoAutoProperties, IgnoreUnknownNodeTypes, null, CommitMessage, filters);

        var hostId = SendMessage(new OnGetDialogHostId());
        
        var context = EngineBackend.Instance.SimpleContext(hostId);

        // var context = SendMessage(new OnGetContext()).Response;
        
        await context.Import(options);
        
        Ok();
    }
}
