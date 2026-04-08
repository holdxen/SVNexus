using System;
using CommunityToolkit.Mvvm.Messaging;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

// public class OnAddTab(MainWindowViewModel.TabItemViewModel value)
//     : ValueChangedMessage<MainWindowViewModel.TabItemViewModel>(value);

public class OnAddTab
{
    public required Func<ViewModelBase, object> Factory { get; set; }

    public string Name { get; set; } = string.Empty;
    
    public bool Closable { get; set; }
}