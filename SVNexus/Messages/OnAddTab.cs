using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnAddTab(MainWindowViewModel.TabItemViewViewModel value) : ValueChangedMessage<MainWindowViewModel.TabItemViewViewModel>(value);