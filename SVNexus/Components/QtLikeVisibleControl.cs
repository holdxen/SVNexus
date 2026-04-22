using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.VisualTree;

namespace SVNexus.Components;

public class QtLikeVisibleControl : UserControl
{
    private bool _lastEffectiveVisible;
    private readonly List<Visual> _watched = [];

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        HookAncestors();
        CheckEffectiveVisibility();
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        UnhookAncestors();
        base.OnDetachedFromVisualTree(e);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);

        if (change.Property == IsVisibleProperty)
            CheckEffectiveVisibility();
    }

    private void HookAncestors()
    {
        UnhookAncestors();

        for (var p = this.GetVisualParent(); p != null; p = p.GetVisualParent())
        {
            p.PropertyChanged += AncestorPropertyChanged;
            _watched.Add(p);
        }
    }

    private void UnhookAncestors()
    {
        foreach (var v in _watched)
            v.PropertyChanged -= AncestorPropertyChanged;
        _watched.Clear();
    }

    private void AncestorPropertyChanged(object? sender, AvaloniaPropertyChangedEventArgs e)
    {
        if (e.Property == IsVisibleProperty)
            CheckEffectiveVisibility();
    }

    private void CheckEffectiveVisibility()
    {
        var now = IsEffectivelyVisible;
        if (now == _lastEffectiveVisible)
            return;

        _lastEffectiveVisible = now;

        if (now)
            OnShownLikeQt();
        else
            OnHiddenLikeQt();
    }

    protected virtual void OnShownLikeQt()
    {
        RaiseEvent(new RoutedEventArgs(ShowEvent));
    }

    protected virtual void OnHiddenLikeQt()
    {
        RaiseEvent(new RoutedEventArgs(HideEvent));
    }
    
    public static readonly RoutedEvent<RoutedEventArgs> ShowEvent =
        RoutedEvent.Register<QtLikeVisibleControl, RoutedEventArgs>(
            nameof(Show),
            RoutingStrategies.Direct);

    public event EventHandler<RoutedEventArgs>? Show
    {
        add => AddHandler(ShowEvent, value);
        remove => RemoveHandler(ShowEvent, value);
    }
    
    public static readonly RoutedEvent<RoutedEventArgs> HideEvent =
        RoutedEvent.Register<QtLikeVisibleControl, RoutedEventArgs>(
            nameof(Hide),
            RoutingStrategies.Direct);

    public event EventHandler<RoutedEventArgs>? Hide
    {
        add => AddHandler(HideEvent, value);
        remove => RemoveHandler(HideEvent, value);
    }

}
