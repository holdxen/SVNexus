using System;
using CommunityToolkit.Mvvm.Messaging.Messages;

namespace SVNexus.Messages;

// public class OnRemoveTab(MainWindowViewModel.TabItemViewViewModel value) : ValueChangedMessage<MainWindowViewModel.TabItemViewViewModel>(value);
//
// public class OnRemoveTabByLocalViewModel(LocalViewModel value): ValueChangedMessage<LocalViewModel>(value);
//
// public class OnRemoveTabByContent(object value) : ValueChangedMessage<object>(value);

public class OnRemoveTab(Guid guid): ValueChangedMessage<Guid>(guid);
