using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;

namespace SVNexus.ViewModels;

public interface IRelativeViewModel
{
    
}

public abstract class ViewModelBase (ViewModelBase? parent = null) : ObservableObject
{
    public virtual Type? ViewType { get; set; }
    
    public ViewModelBase? Parent { get; set; } = parent;
    
    protected ViewModelBase? Sender { get; set; }
    
    private const int MaxLevel = 200;

    public T? GetParent<T>()
        where T: ViewModelBase
    {
        var parent = Parent;
        var times = 0;
        while (parent is not null)
        {
            if (parent is T p)
            {
                return p;
            }
            parent = parent.Parent;
            if (times++ > MaxLevel)
            {
                throw new InvalidOperationException("Too many levels for relationship");
            }
        }
        return null;
    }

    public ViewModelBase? GetRoot()
    {
        var times = 0;
        var parent = Parent;
        while (parent is not null)
        {
            if (parent.Parent is null)
            {
                return parent;
            }
            
            parent = parent.Parent;
            if (times++ > MaxLevel)
            {
                throw new InvalidOperationException("Too many levels for relationship");
            }
        }

        return null;
    }

    public T SendMessage<T>(T message)
        where T: class
    {
        var times = 0;
        var parent = Parent;
        while (parent is not null)
        {
            if (parent is IRecipient<T> recipient)
            {
                parent.Sender = this;
                recipient.Receive(message);
                parent.Sender = null;
            }
            parent = parent.Parent;
            if (times++ > MaxLevel)
            {
                throw new InvalidOperationException("Too many levels for relationship");
            }
        }
        return message;
    }

    //
    // [ObservableProperty]
    // public partial Guid Token { get; set; } = Guid.Empty;
    //
    // partial void OnTokenChanged(Guid oldValue, Guid newValue)
    // {
    //     if (oldValue != Guid.Empty)
    //     {
    //         OnUnregisterMessages();
    //     }
    //
    //     if (newValue != Guid.Empty)
    //     {
    //         OnRegisterMessages(newValue);
    //     }
    // }
    //
    // protected virtual void OnUnregisterMessages()
    // {
    //     Manager.Default.UnregisterAllMessages(this);
    // }
    //
    // protected virtual void OnRegisterMessages(Guid token)
    // {
    //     Manager.Default.RegisterAllMessages(this, token);
    // }
    //
    //
    // private Channel<object>? LoadedActionChannel { get; set; } =  Channel.CreateUnbounded<object>(new UnboundedChannelOptions()
    // {
    //     SingleReader = true,
    //     SingleWriter =  true
    // });
    //
    // public bool Loaded => LoadedActionChannel == null;
    //
    // [RelayCommand]
    // protected virtual async Task OnLoaded()
    // {
    //     while (LoadedActionChannel?.Reader.TryRead(out var obj) ?? false)
    //     {
    //         switch (obj)
    //         {
    //             case Action action:
    //                 action();
    //                 break;
    //             case Func<Task> func:
    //                 await func();
    //                 break;
    //             case Task task:
    //                 await task;
    //                 break;
    //         }
    //     }
    //     LoadedActionChannel = null;
    // }
    //
    // public void InvokeLoadedAction(Action action)
    // {
    //     if (LoadedActionChannel is null)
    //     {
    //         action();
    //     }
    //     else
    //     {
    //         LoadedActionChannel.Writer.TryWrite(action);
    //     }
    // }
    //
    // public async Task InvokeLoadedFunc(Func<Task> action)
    // {
    //     if (LoadedActionChannel is null)
    //     {
    //         await action();
    //     }
    //     else 
    //     {
    //         await LoadedActionChannel.Writer.WriteAsync(action);
    //     }
    // }
}

public abstract partial class ViewModelMore(ViewModelBase? parent = null) : ViewModelBase(parent)
{

    public bool IsLoaded { get; private set; }

    [RelayCommand]
    private async Task OnLoaded()
    {
        if (IsLoaded)
        {
            return;
        }
        try
        {
            await LoadOnce();
        }
        finally
        {
            IsLoaded = true;
        }
    }

    protected virtual Task LoadOnce()
    {
        return Task.CompletedTask;
    }
}
