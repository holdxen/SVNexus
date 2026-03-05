using System;
using CommunityToolkit.Mvvm.Messaging;

namespace SVNexus.Messages;

internal static class RecipientExtensions
{
    extension<T>(IRecipient<T> recipient) where T: class
    {
        public void Register(WeakReferenceMessenger messenger)
        {
            messenger.Register(recipient);
        }

        public void Unregister(WeakReferenceMessenger messenger)
        {
            messenger.Unregister<T>(recipient);
        }
    }
}

public static class Manager
{
    public static readonly WeakReferenceMessenger MainWindow = new();
    
    public static readonly WeakReferenceMessenger MainWindowViewModel = new();

    extension(object target)
    {
        public void RegisterAllMessages(WeakReferenceMessenger messenger)
        {
            (target as IRecipient<OnAddTab>)?.Register(messenger);
            (target as IRecipient<OnCheckout>)?.Register(messenger);
            (target as IRecipient<OnFolderPickerOpen>)?.Register(messenger);
            (target as IRecipient<OnGetDialogHostId>)?.Register(messenger);
            (target as IRecipient<OnInitializeRepository>)?.Register(messenger);
            (target as IRecipient<OnNotification>)?.Register(messenger);
            (target as IRecipient<OnOpenRepository>)?.Register(messenger);
            (target as IRecipient<OnRemoveTabByLocalViewModel>)?.Register(messenger);
            (target as IRecipient<OnRemoveTab>)?.Register(messenger);
            (target as IRecipient<OnWorkingCopyViewEnabled>)?.Register(messenger);
            (target as IRecipient<OnCancel>)?.Register(messenger);
            (target as IRecipient<OnExport>)?.Register(messenger);
            (target as IRecipient<OnNotWorkingCopy>)?.Register(messenger);
            (target as IRecipient<OnRemoveTabByContent>)?.Register(messenger);
            (target as IRecipient<OnSelectedItem>)?.Register(messenger);
        }

        public void UnregisterAllMessages(WeakReferenceMessenger messenger)
        {
            (target as IRecipient<OnAddTab>)?.Unregister(messenger);
            (target as IRecipient<OnCheckout>)?.Unregister(messenger);
            (target as IRecipient<OnFolderPickerOpen>)?.Unregister(messenger);
            (target as IRecipient<OnGetDialogHostId>)?.Unregister(messenger);
            (target as IRecipient<OnInitializeRepository>)?.Unregister(messenger);
            (target as IRecipient<OnNotification>)?.Unregister(messenger);
            (target as IRecipient<OnOpenRepository>)?.Unregister(messenger);
            (target as IRecipient<OnRemoveTabByLocalViewModel>)?.Unregister(messenger);
            (target as IRecipient<OnRemoveTab>)?.Unregister(messenger);
            (target as IRecipient<OnWorkingCopyViewEnabled>)?.Unregister(messenger);
            (target as IRecipient<OnCancel>)?.Unregister(messenger);
            (target as IRecipient<OnExport>)?.Unregister(messenger);
            (target as IRecipient<OnNotWorkingCopy>)?.Unregister(messenger);
            (target as IRecipient<OnRemoveTabByContent>)?.Unregister(messenger);
            (target as IRecipient<OnSelectedItem>)?.Unregister(messenger);
        }
    }
}