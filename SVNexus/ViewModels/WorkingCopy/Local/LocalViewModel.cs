using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Text.Json;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalViewModel: ViewModelBase
{

    [ObservableProperty]
    private bool _isTreeView = true;

    [ObservableProperty]
    private bool _isListView;


    public required LocalListViewModel LocalListViewModel { get; set; }
    
    public required LocalTreeViewModel LocalTreeViewModel { get; set; }


    public override bool KeepAlive { get; set; } = true;


    [ObservableProperty]
    private string _commitMessage = string.Empty;



    [ObservableProperty]
    private LocalTreeViewModel.TreeItemViewModel? _treeSelectedItem;
    
    [ObservableProperty]
    private LocalListViewModel.ListItemViewModel? _listSelectedItem;
    
    public ObservableCollection<LocalListViewModel.ListItemViewModel> LocalListItems { get; set; } = [];
    
    
    public ObservableCollection<LocalTreeViewModel.TreeItemViewModel> LocalTreeItems { get; set; } = [];
    
    
    public required string WorkingCopyPath { get; init; }
    
    public required WeakReferenceMessenger Messenger { get; init; }


    [RelayCommand]
    private async Task OnLoaded()
    {
        await Status();
    }

    public static LocalViewModel Create(string workingCopyPath, WeakReferenceMessenger workingCopyViewMessenger)
    {
        var model = new LocalViewModel
        {
            Messenger = workingCopyViewMessenger,
            LocalListViewModel = new LocalListViewModel()
            {
                WorkingCopyPath = workingCopyPath,
            },
            LocalTreeViewModel = new LocalTreeViewModel()
            {
                WorkingCopyPath =  workingCopyPath,
            },
            WorkingCopyPath = workingCopyPath,
        };
        return model;
    }

    // public void Initialize()
    // {
    //     LocalListViewModel = new LocalListViewModel
    //     {
    //         WorkingCopyPath = WorkingCopyPath,
    //     };
    //
    //     LocalTreeViewModel = new LocalTreeViewModel
    //     {
    //         WorkingCopyPath = WorkingCopyPath,
    //     };
    //
    // }

    [RelayCommand]
    private async Task Status()
    {
        Console.WriteLine($"WorkingCopyPath: {WorkingCopyPath}");
        var contextNotifier = new ContextNotifierDelegate
        {
            
        };
        Console.WriteLine($"create");
        
        var createContextOptions = Engine.Engine.Instance.MakeCreateContextOptions(contextNotifier);
        
        Console.WriteLine($"contex5");
        try
        {

            using var context = AsyncContext.Create(createContextOptions);


            Console.WriteLine($"opt");
            var statusOptions = new StatusOptions(Path: WorkingCopyPath, Revision: new Revision.Working(), Depth: Depth.Infinity, GetAll: false, Update: true, CheckOutOfDate: false, CheckWorkingCopy: false, NoIgnore: false, IgnoreExternals: true, DepthAsSticky: false, Changelist:[]);


            Console.WriteLine($"exec");
            var result = await context.Status(statusOptions);
        
            Console.WriteLine("Result: entry: {0}", result.Entries.Length);

            Update(result.Entries);
        }
        catch (System.Exception e)
        {
            var options = new JsonSerializerOptions
            {
                PropertyNameCaseInsensitive = true,
                PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
            };
            if (e is Generated.Exception.SvnException svnException)
            {
                var error = JsonSerializer.Deserialize<SvnError>(svnException.Message, options)!;
                var errorNumber = new SvnErrnoConstants();
                if (errorNumber.IsWcNotDirectory(error.Code))
                {
                    Console.WriteLine($"WcNotDirectory: {error.Code}");
                    var hostId = Messenger.Send(new OnGetDialogHostId()).Response;
                    var result = await MessageBox.ShowOverlayAsync(title: "Error", hostId: hostId, message: "Not a working copy, initialize now", button: MessageBoxButton.YesNo);
                    if (result is MessageBoxResult.No)
                    {
                        WeakReferenceMessenger.Default.Send(new OnRemoveTabByLocalViewModel(this));
                    }
                    else
                    {
                        var dialogOptions = new OverlayDialogOptions
                        {
                            Title = "Test",
                            IsCloseButtonVisible = true,
                            Buttons = DialogButton.None
                        };
                        await OverlayDialog.ShowModal<ImportDialog, ImportDialogModel>(new ImportDialogModel()
                        {
                            Messenger = Messenger,
                            Path = WorkingCopyPath,
                        }, hostId: hostId, options: dialogOptions);
                    }
                }
            }

            Messenger.Send(new OnWorkingCopyViewEnabled(false));
        }
    }


    private void Update(StatusEntry[] statusEntries)
    {
        LocalListViewModel.Update(statusEntries);
        
        LocalTreeViewModel.Update(statusEntries);
    }

    public Type DepthType => typeof(Depth);
    

    
    
    
    [ObservableProperty]
    public partial Depth Depth { get; set; } = Depth.Infinity;


    [RelayCommand]
    private async Task Commit()
    {
        var hostId = Messenger.Send(new OnGetDialogHostId()).Response;


        var contextNotifierDelegate = new ContextNotifierDelegate()
        {
            DialogHostId = hostId
        };


        var createContextOptions = Engine.Engine.Instance.MakeCreateContextOptions(contextNotifierDelegate);

        var context = AsyncContext.Create(createContextOptions);


        var commitOptions = new CommitOptions(
            Targets: IsTreeView
                ? LocalTreeViewModel.SelectedItems.ToArray()
                : LocalListViewModel.SelectedItems.ToArray(),
            Depth: Depth,
            KeepLocks: true,
            KeepChangelist: false, CommitAsOperations: true, IncludeFileExternals: true, IncludeDirExternals: true,
            Changelists: [], RevisionPropertyTable: new Dictionary<string, string>(), CommitMessage: CommitMessage);
        
        await context.Commit(commitOptions);

    }
}