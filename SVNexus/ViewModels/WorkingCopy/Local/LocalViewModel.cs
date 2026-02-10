using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Engine;
using SVNexus.Generated;

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
    private LocalListViewModel.ListItemViewModel ? _listSelectedItem;
    
    public ObservableCollection<LocalListViewModel.ListItemViewModel> LocalListItems { get; set; } = [];
    
    
    public ObservableCollection<LocalTreeViewModel.TreeItemViewModel> LocalTreeItems { get; set; } = [];
    
    
    public required string WorkingCopyPath { get; init; }


    [RelayCommand]
    private async Task OnLoaded()
    {
        await Status();
    }

    public static LocalViewModel Create(string workingCopyPath)
    {
        var model = new LocalViewModel
        {
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
            Console.WriteLine(e);
            throw;
        }
    }


    private void Update(StatusEntry[] statusEntries)
    {
        LocalListViewModel?.Update(statusEntries);
    }




    [RelayCommand]
    private void Commit()
    {
        
    }
}