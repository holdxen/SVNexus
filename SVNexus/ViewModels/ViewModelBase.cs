using System;
using CommunityToolkit.Mvvm.ComponentModel;
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
            Console.WriteLine("RegisterAllMessages: Token: {0} {1}", Token, this.GetType().FullName);
            Manager.Default.RegisterAllMessages(this, newValue);
        }
    }
}