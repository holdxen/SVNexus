using CommunityToolkit.Mvvm.Messaging;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnAddTab(MainWindowViewModel.TabItemViewModel value)
    : ValueChangedMessage<MainWindowViewModel.TabItemViewModel>(value);