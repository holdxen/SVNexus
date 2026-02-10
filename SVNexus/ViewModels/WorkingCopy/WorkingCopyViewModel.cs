using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.ViewModels.WorkingCopy.Remote;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkingCopyViewModel : ViewModelBase
{
    public override bool KeepAlive { get; set; } = true;

    public required string WorkingCopyPath { get; set; }
    
    [ObservableProperty] public partial bool IsLocalView { get; set; } = true;

    [ObservableProperty] public partial bool IsRemoteView { get; set; } = false;
    
    
    [ObservableProperty]
    public required partial LocalViewModel LocalViewModel { get; set; }
    
    
    [ObservableProperty]
    public required partial RemoteViewModel RemoteViewModel { get; set; }


    public static WorkingCopyViewModel Create(string workingCopyPath)
    {
        return new WorkingCopyViewModel
        {
            WorkingCopyPath = workingCopyPath,
            LocalViewModel = LocalViewModel.Create(workingCopyPath),
            RemoteViewModel = new RemoteViewModel()
        };
    }

    // public void Initialize()
    // {
    //     LocalViewModel ??= new LocalViewModel
    //     {
    //         WorkingCopyPath = WorkingCopyPath
    //     };
    //     
    //     // LocalViewModel.Initialize();
    //
    //     RemoteViewModel ??= new RemoteViewModel() { };
    // }

}