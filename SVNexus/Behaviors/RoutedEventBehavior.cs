using System;
using System.Linq;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.VisualTree;

namespace SVNexus.Behaviors;

using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Xaml.Interactivity;

public class RoutedEventBehavior : Behavior<Interactive>
{
    public static readonly StyledProperty<string?> NameProperty = AvaloniaProperty.Register<RoutedEventBehavior, string?>(
        nameof(Name));

    public string? Name
    {
        get => GetValue(NameProperty);
        set => SetValue(NameProperty, value);
    }
    public static readonly StyledProperty<bool> PreventProperty = AvaloniaProperty.Register<RoutedEventBehavior, bool>(
        nameof(Prevent));

    public bool Prevent
    {
        get => GetValue(PreventProperty);
        set => SetValue(PreventProperty, value);
    }
    
    public static readonly StyledProperty<RoutedEvent?> RoutedEventProperty =
        AvaloniaProperty.Register<RoutedEventBehavior, RoutedEvent?>(nameof(RoutedEvent));

    public static readonly StyledProperty<RoutingStrategies> RoutingStrategiesProperty =
        AvaloniaProperty.Register<RoutedEventBehavior, RoutingStrategies>(
            nameof(RoutingStrategies),
            RoutingStrategies.Bubble);


    public static readonly StyledProperty<bool> ShowFlyoutProperty = AvaloniaProperty.Register<RoutedEventBehavior, bool>(
        nameof(ShowFlyout));

    public bool ShowFlyout
    {
        get => GetValue(ShowFlyoutProperty);
        set => SetValue(ShowFlyoutProperty, value);
    }

    public RoutedEvent? RoutedEvent
    {
        get => GetValue(RoutedEventProperty);
        set => SetValue(RoutedEventProperty, value);
    }

    public RoutingStrategies RoutingStrategies
    {
        get => GetValue(RoutingStrategiesProperty);
        set => SetValue(RoutingStrategiesProperty, value);
    }

    protected override void OnAttached()
    {
        base.OnAttached();

        if (AssociatedObject != null && RoutedEvent != null)
        {
            AssociatedObject.AddHandler(RoutedEvent, OnRoutedEvent, RoutingStrategies);
        }
    }

    protected override void OnDetaching()
    {
        if (AssociatedObject != null && RoutedEvent != null)
        {
            AssociatedObject.RemoveHandler(RoutedEvent, OnRoutedEvent);
        }

        base.OnDetaching();
    }

    private void OnRoutedEvent(object? sender, RoutedEventArgs e)
    {
        Console.WriteLine("Show Flyout: {0}, sender={1}", ShowFlyout, sender);
        if (ShowFlyout && sender is Control control)
        {
            FlyoutBase.ShowAttachedFlyout(control);
        }
        e.Handled = Prevent;
    }
}
