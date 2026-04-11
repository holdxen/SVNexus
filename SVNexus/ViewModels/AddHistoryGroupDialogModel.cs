using System;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;

namespace SVNexus.ViewModels;

public partial class AddHistoryGroupDialogModel: ViewModelMore, IDialogContext
{
    [ObservableProperty]
    public partial string Name { get; set; } = string.Empty;
    
    

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;

    
    public WorkspaceHistoryGroup? HistoryGroup { get; private set; }

    [RelayCommand]
    private async Task Confirm()
    {
        await EngineBackend.Instance.DatabaseQueue.RunAndWait(async _ =>
        {
            var groups = await SeaDatabaseConnection.Default.HistoryGroups();
            if (groups.FindIndex(g => g.Name == Name) != -1)
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = $"Group {Name} already exists",
                    Type = NotificationType.Error
                }, Manager.MainWindowToken);
                return;
            }

            var group = new WorkspaceHistoryGroup(Guid.NewGuid().ToString(), Name, []);
            
            await SeaDatabaseConnection.Default.AddHistoryGroup(group);
            
            RequestClose?.Invoke(this, null);
            
            HistoryGroup = group;
            
        });
    }
}