using System;
using System.IO;
using System.Text;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Platform.Storage;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;

namespace SVNexus.ViewModels;

public partial class DifferenceDialogModel : ViewModelMore, IDialogContext
{
    
    public enum DisplayContent
    {
        All,
        ContentOnly,
        PropertyOnly
    }

    [ObservableProperty]
    public partial string Path { get; set; } = string.Empty;
    
    public static Type DepthType = typeof(Depth);

    [ObservableProperty]
    public partial Depth Depth { get; set; } = Depth.Infinity;

    [ObservableProperty] public partial bool IgnoreAncestry { get; set; } = true;
    
    [ObservableProperty] public partial bool NoAdded { get; set; }
    
    [ObservableProperty] public partial bool NoDeleted { get; set; }
    
    [ObservableProperty] public partial bool IgnoreContentType { get; set; }
    
    public static Type DisplayContentType = typeof(DisplayContent);

    [ObservableProperty]
    public partial DisplayContent Display { get; set; } = DisplayContent.All;
    
    [ObservableProperty]
    public partial bool GitFormat { get; set; }

    [ObservableProperty] public partial bool Pretty { get; set; } = true;
    
    
    [ObservableProperty] public partial bool ShowCopiesAsAdds { get; set; }
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(CanExport))]
    public partial string Out { get; set; } = string.Empty;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(HasWarning))]
    public partial string Err { get; set; } = string.Empty;
    
    public bool HasWarning => !string.IsNullOrEmpty(Err);
    
    public bool CanExport => !string.IsNullOrEmpty(Out) && State is LoadingOrErrorState.None;
    

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(CanExport))]
    public partial LoadingOrErrorState State { get; set; } = LoadingOrErrorState.MakeNone();

    [ObservableProperty] public partial bool RelateToRoot { get; set; } = true;

    private readonly SingleTaskQueue _singleTaskQueue = new();

    /// <inheritdoc/>
    public DifferenceDialogModel(ViewModelBase parent) : base(parent)
    {
        PropertyChanged += (sender, args) =>
        {
            if (args.PropertyName is nameof(RelateToRoot) or nameof(ShowCopiesAsAdds) or nameof(GitFormat) or nameof(Pretty) or nameof(Path) or nameof(Depth) or nameof(IgnoreAncestry) or nameof(NoAdded) or nameof(NoDeleted) or nameof(IgnoreContentType) or nameof(Display)) 
            {
                _singleTaskQueue.Run(Execute);
            }
        };
    }

    private async Task Execute(CancellationToken token)
    {
        token.ThrowIfCancellationRequested();
        State = LoadingOrErrorState.MakeLoading();
        var context = SendMessage(new OnGetContext()).Response;

        try
        {

            var differenceOptions = new ClientDifferenceOptions(
                null, 
                Path,
                new Revision.Base(), 
                Path, new Revision.Working(), 
                RelateToRoot ? SendMessage(new OnGetWorkingCopyRoot()).Response : null, 
                Depth, 
                IgnoreAncestry, 
                NoAdded, 
                NoDeleted, 
                ShowCopiesAsAdds, 
                IgnoreContentType, 
                Display == DisplayContent.ContentOnly, 
                Display == DisplayContent.PropertyOnly, 
                GitFormat, 
                Pretty, 
                "UTF-8", 
                null);
        
        
            var result = await context.Difference(differenceOptions);
            token.ThrowIfCancellationRequested();

            Err = Encoding.UTF8.GetString(result.Err);
        
            Out = Encoding.UTF8.GetString(result.Out);
        }
        catch (System.Exception e) when (e is not OperationCanceledException)
        {
            State = LoadingOrErrorState.MakeError(e.HumanReadableMessage);
            return;
        }

        token.ThrowIfCancellationRequested();


        State = LoadingOrErrorState.MakeNone();

    }
    
    protected override async Task LoadOnce()
    {
        await _singleTaskQueue.Run(Execute);
    }


    [RelayCommand]
    private async Task Export()
    {
        var saveFileOptions = new OnFilePickerSave()
        {
            Options = new FilePickerSaveOptions()
            {
                Title = "Export as a file",
                ShowOverwritePrompt = true,
                SuggestedFileName = "test.txt"
            }
        };
        
        var file = await Manager.Default.Send(saveFileOptions, Manager.MainWindowToken);
        if (file is null)
        {
            return;
        }

        await File.WriteAllTextAsync(file.Path.AbsolutePath, Out);
    }

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;
}