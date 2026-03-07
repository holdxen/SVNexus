using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Text.Json;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Components;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public class DifferenceInfo
{
    public required List<string> Original { get; set; }
    public required List<string> Modified { get; set; }
    public required TextChange[] Changes { get; set; }
}

public partial class LocalViewModel: ViewModelBase, IRecipient<OnSelectedItemChanged>
{

    private Dictionary<string, DifferenceInfo> _differenceInfos = [];

    [ObservableProperty]
    private bool _isTreeView = true;

    [ObservableProperty]
    private bool _isListView;


    public LocalListViewModel LocalListViewModel { get; set; } = new();

    public LocalTreeViewModel LocalTreeViewModel { get; set; } = new();


    public override bool KeepAlive { get; set; } = true;


    [ObservableProperty]
    private string _commitMessage = string.Empty;



    [ObservableProperty]
    private LocalTreeViewModel.TreeItemViewModel? _treeSelectedItem;
    
    [ObservableProperty]
    private LocalListViewModel.ListItemViewModel? _listSelectedItem;
    
    // public ObservableCollection<LocalListViewModel.ListItemViewModel> LocalListItems { get; set; } = [];
    //
    //
    // public ObservableCollection<LocalTreeViewModel.TreeItemViewModel> LocalTreeItems { get; set; } = [];


    [ObservableProperty]
    public partial string WorkingCopyPath { private get; set; } = string.Empty;
    
    // public required WeakReferenceMessenger Messenger { get; init; }

    [ObservableProperty] public partial LoadingOrErrorState EditorState { get; set; } = LoadingOrErrorState.MakeNone();

    [ObservableProperty] public partial string Code { get; set; } = string.Empty;


    [RelayCommand]
    private async Task OnLoaded()
    {
        await Status();
    }

    // public static LocalViewModel Create(WeakReferenceMessenger workingCopyViewMessenger)
    // {
    //     var model = new LocalViewModel
    //     {
    //         // Messenger = workingCopyViewMessenger,
    //         LocalListViewModel = new LocalListViewModel(),
    //         LocalTreeViewModel = new LocalTreeViewModel(),
    //     };
    //     
    //     model.LocalListViewModel.SelectedItemChanged += OnSelectedItemChanged;
    //     model.LocalTreeViewModel.SelectedItemChanged += OnSelectedItemChanged;
    //     
    //     return model;
    // }

    private static void OnSelectedItemChanged(object? arg1, StatusEntry? entry)
    {
        throw new NotImplementedException();
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
    public async Task Status()
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
            e.Handle(svnExceptionHandler: error =>
            {
                var errorNumber = new SvnErrnoConstants();
                if (errorNumber.IsWcNotDirectory(error.Code))
                {
                    Manager.Default.Send(new OnNotWorkingCopy(WorkingCopyPath), Token);
                }
            });
        }
    }


    private void Update(StatusEntry[] statusEntries)
    {
        LocalListViewModel.Update(statusEntries);
        
        LocalTreeViewModel.Update(statusEntries);
    }

    public static Type DepthType => typeof(Depth);
    

    
    
    
    [ObservableProperty]
    public partial Depth Depth { get; set; } = Depth.Infinity;


    [RelayCommand]
    private async Task Commit()
    {
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;


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
            KeepChangelist: false, 
            CommitAsOperations: true, 
            IncludeFileExternals: true, 
            IncludeDirExternals: true,
            Changelists: [], 
            RevisionPropertyTable: new Dictionary<string, string>(), 
            CommitMessage: CommitMessage);
        
        await context.Commit(commitOptions);

    }

    public void Receive(OnSelectedItemChanged message)
    {
    }
}