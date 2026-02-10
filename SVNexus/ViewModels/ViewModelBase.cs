using System;
using CommunityToolkit.Mvvm.ComponentModel;

namespace SVNexus.ViewModels;

public abstract class ViewModelBase : ObservableObject
{
    public virtual bool KeepAlive { get; set; }
    
    public virtual Type? ViewType { get; set; }
}