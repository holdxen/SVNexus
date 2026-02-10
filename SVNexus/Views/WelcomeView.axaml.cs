using System;
using Avalonia;
using Avalonia.Controls;
using Ursa.Controls;

namespace SVNexus.Views;

public partial class WelcomeView : UserControl
{
    public static readonly StyledProperty<string> DialogHostIdProperty = AvaloniaProperty.Register<WelcomeView, string>(
        nameof(DialogHostId));
    public string DialogHostId
    {
        get => GetValue(DialogHostIdProperty);
        private set => SetValue(DialogHostIdProperty, value);
    }
    
    public WelcomeView()
    {
        InitializeComponent();
        var hostId = Guid.NewGuid().ToString();
        this.FindControl<OverlayDialogHost>("WelcomeViewDialogHost")!.HostId = hostId;
        
        DialogHostId = hostId;
    }
}