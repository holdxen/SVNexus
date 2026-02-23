using CommunityToolkit.Mvvm.Messaging;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.ViewModels;

namespace SVNexus.Messages;

public class OnAddTab(MainWindowViewModel.TabItemViewViewModel value)
    : ValueChangedMessage<MainWindowViewModel.TabItemViewViewModel>(value)
{
    public static void Register(WeakReferenceMessenger messenger, object target)
    {
        if (target is IRecipient<OnAddTab> recipient)
        {
            messenger.Register(recipient);
        }
    }
}