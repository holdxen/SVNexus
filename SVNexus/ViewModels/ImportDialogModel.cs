using System;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class ImportDialogModel: ViewModelBase, IDialogContext
{
    public override Type? ViewType { get; set; } = typeof(ImportDialog);
    
    [ObservableProperty]
    public partial string Url { get; set; } = string.Empty;



    [ObservableProperty] public partial bool Backup { get; set; } = true;
    
    
    public string Path { get; set; } = string.Empty;
    
    public InitializeRepositoryOptions? Options { get; set; }

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;



    [RelayCommand]
    private void Confirm()
    {
        Options = new InitializeRepositoryOptions(Backup: Backup, Local: Path, Remote: Url);
        
        
        Close();
    }
}
