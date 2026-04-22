using System;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy.Changes;
using SVNexus.ViewModels.WorkingCopy.History;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkingCopyViewModel : ViewModelBase,
    IRecipient<OnWorkingCopyViewEnabled>, 
    // IRecipient<OnNotWorkingCopy>,
    IRecipient<OnGetWorkingCopyPath>
{

    public const int LocalViewIndex = 0;
    public const int ChangesViewIndex = 1;
    public const int HistoryViewIndex = 2;
    public const int RemoteViewIndex = 3;
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsLocalView))]
    [NotifyPropertyChangedFor(nameof(IsRemoteView))]
    [NotifyPropertyChangedFor(nameof(IsChangesView))]
    [NotifyPropertyChangedFor(nameof(IsHistoryView))]
    public partial int SelectedViewIndex { get; set; } = ChangesViewIndex;

    // [ObservableProperty]
    // public required partial string WorkingCopyPath { get; set; }
    
    public bool IsLocalView => SelectedViewIndex == LocalViewIndex;

    public bool IsRemoteView => SelectedViewIndex == RemoteViewIndex;
    
    public bool IsHistoryView => SelectedViewIndex == HistoryViewIndex;
    
    public bool IsChangesView => SelectedViewIndex == ChangesViewIndex;
    
    
    public ChangesViewModel ChangesViewModel { get; }

    public HistoryViewModel HistoryViewModel { get; }
    
    
    public LocalViewModel LocalViewModel { get; } 
    
    public string Path { get; }
    
    [ObservableProperty] public partial bool IsEnabled { get; set; } = true;

    //
    // partial void OnSelectedViewIndexChanged(int value)
    // {
    //     switch (value)
    //     {
    //         case ChangesViewIndex:
    //             Dispatcher.UIThread.InvokeAsync(async () =>
    //             {
    //                 await ChangesViewModel.Status();
    //             });
    //             break;
    //         case LocalViewIndex:
    //             Dispatcher.UIThread.InvokeAsync(async () =>
    //             {
    //                 await LocalViewModel.RefreshCommand.ExecuteOrNothingAsync(null);
    //             });
    //             break;
    //     }
    // }

    [RelayCommand]
    private async Task Update()
    {
        try
        {
            var context = SendMessage(new OnGetContext()).Response;
            
            var opts = new UpdateOptions(Paths: [Path], Revision: new Revision.Head(), Depth.Infinity, DepthIsSticky: false, IgnoreExternals: false, AllowUnverObstructions: true, AddsAsModification: false, MakeParents: true);
            
            await context.Update(opts);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to update working copy: {e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
    }

    

    // private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    // private readonly Services.ITabService  _tabService;
    // private readonly Services.TypeService _typeService;
    //
    // public WorkingCopyViewModel(IServiceProvider serviceProvider)
    // {
    //     _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
    //     _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
    //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
    //     
    //     // LocalViewModel = serviceProvider.GetRequiredService<LocalViewModel>();
    //     LocalViewModel = new LocalViewModel(this);
    //     
    //     // ChangesViewModel = serviceProvider.GetRequiredService<ChangesViewModel>();
    //     ChangesViewModel = new ChangesViewModel(this);
    //     
    //     HistoryViewModel = serviceProvider.GetRequiredService<HistoryViewModel>();
    //     
    //     Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
    // }

    public WorkingCopyViewModel(ViewModelBase parent, string path): base(parent)
    {
        Path = path;
        ChangesViewModel = new ChangesViewModel(this);
        HistoryViewModel = new HistoryViewModel(this);
        LocalViewModel = new LocalViewModel(this);
    }

    public void Receive(OnWorkingCopyViewEnabled message)
    {
        IsEnabled = message.Value;
    }

    public void Receive(InitializeRepositoryOptions message)
    {
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId());

        var importProcessDialogModel = new ImportProcessDialogModel(this)
        {
            Options = message,
        };

        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Initialize repository",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };

            await OverlayDialog.ShowModal<ImportProcessDialog, ImportProcessDialogModel>(importProcessDialogModel, options: options, hostId: hostId);
            if (importProcessDialogModel.Error is null)
            {
                await ChangesViewModel.Status();
            }
        });

    }



    public void Receive(OnGetWorkingCopyPath message)
    {
        message.Reply(Path);
    }
}