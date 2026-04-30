using System.ComponentModel;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Xaml.Interactivity;

namespace SVNexus.Behaviors;

public class ContextMenuControlBehavior: Behavior<Control>
{
    public static readonly StyledProperty<bool> AllowToOpenProperty = AvaloniaProperty.Register<ContextMenuBehavior, bool>(
        nameof(AllowToOpen));

    public bool AllowToOpen
    {
        get => GetValue(AllowToOpenProperty);
        set => SetValue(AllowToOpenProperty, value);
    }

    private Control? _control;
    
    protected override void OnAttached()
    {
        base.OnAttached();
        _control = AssociatedObject;
        _control?.ContextRequested += ContextMenuOnOpening;
    }

    private void ContextMenuOnOpening(object? sender, ContextRequestedEventArgs e)
    {
        e.Handled = !AllowToOpen;
    }

    protected override void OnDetaching()
    {
        base.OnDetaching();
        _control?.ContextRequested -= ContextMenuOnOpening;
    }
}