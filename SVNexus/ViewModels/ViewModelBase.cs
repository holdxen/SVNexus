using System;
using System.Collections.Generic;
using System.Threading.Channels;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Messages;

namespace SVNexus.ViewModels;

public abstract partial class ViewModelBase : ObservableObject
{
    public virtual bool KeepAlive { get; set; }
    
    public virtual Type? ViewType { get; set; }


    [ObservableProperty]
    public partial Guid Token { get; set; } = Guid.Empty;

    partial void OnTokenChanged(Guid oldValue, Guid newValue)
    {
        if (oldValue != Guid.Empty)
        {
            Manager.Default.UnregisterAllMessages(this);
        }

        if (newValue != Guid.Empty)
        {
            Manager.Default.RegisterAllMessages(this, newValue);
        }
    }
    

    private Channel<Func<Task>>? LoadedActionChannel { get; set; } =  Channel.CreateUnbounded<Func<Task>>(new  UnboundedChannelOptions()
    {
        SingleReader = true,
        SingleWriter =  true
    });

    [RelayCommand]
    protected virtual async Task OnLoaded()
    {
        while (LoadedActionChannel?.Reader.TryRead(out var action) ?? false)
        {
            await action();
        }
        LoadedActionChannel = null;
    }

    public async Task InvokeLoadedAction(Func<Task> action)
    {
        if (LoadedActionChannel is null)
        {
            await action();
        }
        else 
        {
            await LoadedActionChannel.Writer.WriteAsync(action);
        }
    }
}