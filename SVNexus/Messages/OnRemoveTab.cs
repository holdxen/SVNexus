using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy.Local;

namespace SVNexus.Messages;

public class OnRemoveTab(MainWindowViewModel.TabItemViewViewModel value) : ValueChangedMessage<MainWindowViewModel.TabItemViewViewModel>(value);

public class OnRemoveTabByLocalViewModel(LocalViewModel value): ValueChangedMessage<LocalViewModel>(value);

public class OnRemoveTabByContent(object value) : ValueChangedMessage<object>(value);