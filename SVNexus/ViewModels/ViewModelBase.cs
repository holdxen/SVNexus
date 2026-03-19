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
    

    private Channel<object>? LoadedActionChannel { get; set; } =  Channel.CreateUnbounded<object>(new  UnboundedChannelOptions()
    {
        SingleReader = true,
        SingleWriter =  true
    });
    
    public bool Loaded => LoadedActionChannel == null;

    [RelayCommand]
    protected virtual async Task OnLoaded()
    {
        while (LoadedActionChannel?.Reader.TryRead(out var obj) ?? false)
        {
            switch (obj)
            {
                case Action action:
                    action();
                    break;
                case Func<Task> func:
                    await func();
                    break;
                case Task task:
                    await task;
                    break;
            }
        }
        LoadedActionChannel = null;
    }

    public void InvokeLoadedAction(Action action)
    {
        if (LoadedActionChannel is null)
        {
            action();
        }
        else
        {
            LoadedActionChannel.Writer.TryWrite(action);
        }
    }

    public async Task InvokeLoadedFunc(Func<Task> action)
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