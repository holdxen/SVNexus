using System;
using CommunityToolkit.Mvvm.Messaging;

namespace SVNexus.Messages;

internal static class RecipientExtensions
{
    public static void Register<T>(this IRecipient<T> recipient, WeakReferenceMessenger messenger)
        where T: class
    {
        messenger.Register(recipient);
    }
    
    public static void Unregister<T>(this IRecipient<T> recipient, WeakReferenceMessenger messenger)
        where T: class
    {
        messenger.Unregister<T>(recipient);
    }
}

public static class Manager
{

    public static void RegisterAllMessages(this object target, WeakReferenceMessenger messenger)
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
    }
    
    public static void UnregisterAllMessages(this object target, WeakReferenceMessenger messenger)
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
    }
}