using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Xaml.Interactivity;

namespace SVNexus.Behaviors;

public class ContextMenuBehavior: Behavior<ContextMenu>
{

    public static readonly StyledProperty<bool> AllowToOpenProperty = AvaloniaProperty.Register<ContextMenuBehavior, bool>(
        nameof(AllowToOpen));

    public bool AllowToOpen
    {
        get => GetValue(AllowToOpenProperty);
        set => SetValue(AllowToOpenProperty, value);
    }

    private ContextMenu? _contextMenu;
    
    protected override void OnAttached()
    {
        base.OnAttached();
        _contextMenu = AssociatedObject;
        _contextMenu?.Opening += ContextMenuOnOpening;
    }

    private void ContextMenuOnOpening(object? sender, CancelEventArgs e)
    {
        e.Cancel = !AllowToOpen;
    }

    protected override void OnDetaching()
    {
        base.OnDetaching();
        _contextMenu?.Opening -= ContextMenuOnOpening;
    }
}