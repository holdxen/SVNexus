using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class ImportDialogModel: DialogModelBase
{
    
    [ObservableProperty]
    public partial string Url { get; set; } = string.Empty;



    [ObservableProperty] public partial bool Backup { get; set; } = true;
    
    
    public string Path { get; set; } = string.Empty;
    
    public InitializeRepositoryOptions? Options { get; set; }

    protected override Task OnConfirm()
    {
        // Options = new InitializeRepositoryOptions(Backup: Backup, Local: Path, Remote: Url);
        Ok();
        return Task.CompletedTask;
    }
}
