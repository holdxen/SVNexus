using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnRemoveTab(MainWindowViewModel.TabItemViewViewModel value) : ValueChangedMessage<MainWindowViewModel.TabItemViewViewModel>(value);