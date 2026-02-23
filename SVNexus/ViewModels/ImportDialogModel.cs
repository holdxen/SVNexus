using System;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Views;

namespace SVNexus.ViewModels;

public partial class ImportDialogModel: ViewModelBase, IDialogContext
{
    public override Type? ViewType { get; set; } = typeof(ImportDialog);
    
    [ObservableProperty]
    public partial string Url { get; set; } = string.Empty;



    [ObservableProperty] public partial bool Backup { get; set; } = true;
    
    
    public required WeakReferenceMessenger Messenger { get; init; }
    
    public string Path { get; set; } = string.Empty;
    

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
    }

    public event EventHandler<object?>? RequestClose;



    [RelayCommand]
    private void Confirm()
    {
        var options = new InitializeRepositoryOptions(Backup: Backup, Local: Path, Remote: Url);
        
        Messenger.Send(new OnInitializeRepository(options));
        
        Close();
    }
}
