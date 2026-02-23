using System.Threading.Tasks;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.ViewModels.WorkingCopy.Remote;
using SVNexus.Views;
using Tmds.DBus.Protocol;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class WorkingCopyViewModel : ViewModelBase, IRecipient<OnGetDialogHostId>, IRecipient<OnWorkingCopyViewEnabled>, IRecipient<OnInitializeRepository>
{
    public override bool KeepAlive { get; set; } = true;

    public required string WorkingCopyPath { get; set; }
    
    [ObservableProperty] public partial bool IsLocalView { get; set; } = true;

    [ObservableProperty] public partial bool IsRemoteView { get; set; } = false;
    
    
    [ObservableProperty]
    public required partial LocalViewModel LocalViewModel { get; set; }
    
    
    [ObservableProperty]
    public required partial RemoteViewModel RemoteViewModel { get; set; }

    
    [ObservableProperty]
    public partial string? DialogHostId { get; set; }

    public required WeakReferenceMessenger Messenger { get; init; }


    [ObservableProperty] public partial bool IsEnabled { get; set; } = true;

    

    public static WorkingCopyViewModel Create(string workingCopyPath)
    {
        var messenger = new WeakReferenceMessenger();
        return new WorkingCopyViewModel
        {
            Messenger = messenger,
            WorkingCopyPath = workingCopyPath,
            LocalViewModel = LocalViewModel.Create(workingCopyPath, messenger),
            RemoteViewModel = new RemoteViewModel()
        }.RegisterMessages();
    }


    private WorkingCopyViewModel RegisterMessages()
    {
        this.RegisterAllMessages(Messenger);
        return this;
    }
    

    public void Receive(OnGetDialogHostId message)
    {
        message.Reply(DialogHostId);
    }

    public void Receive(OnWorkingCopyViewEnabled message)
    {
        IsEnabled = message.Value;
    }

    public void Receive(OnInitializeRepository message)
    {
        // var contextNotifierDelegate = new ContextNotifierDelegate
        // {
        //
        // };
        //
        //
        // Task.Run(async () =>
        // {
        //     var context = AsyncContext.Create(Engine.Engine.Instance.MakeCreateContextOptions(contextNotifierDelegate));
        //
        //     await context.InitializeRepository(message.Value, new InitializeRepositoryNotifierDelegate()
        //     {
        //     });
        // });


        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Test",
                IsCloseButtonVisible = true,
                Buttons = DialogButton.None
            };

            await OverlayDialog.ShowModal<ImportProcessDialog, ImportProcessDialogModel>(new ImportProcessDialogModel(), options: options);
        });

    }
}