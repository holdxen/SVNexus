using System;
using System.Diagnostics;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using SVNexus.Inject;
using Ursa.Controls;

namespace SVNexus.Views.WorkingCopy;

public partial class WorkingCopyView : UserControl
{
    
    
    // public static readonly StyledProperty<string> DialogHostIdProperty = AvaloniaProperty.Register<WorkingCopyView, string>(
    //     nameof(DialogHostId));
    // public string DialogHostId
    // {
    //     get => GetValue(DialogHostIdProperty);
    //     private set => SetValue(DialogHostIdProperty, value);
    // }
    
    public WorkingCopyView()
    {
        InitializeComponent();
        var host = this.FindControl<OverlayDialogHost>("WorkingCopyViewDialogHost")!;
        host.HostId = Guid.NewGuid().ToString();
        
        Ambient.SetDialogHostId(this, host.HostId);
    }
}