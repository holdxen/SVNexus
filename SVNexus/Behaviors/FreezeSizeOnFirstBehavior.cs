using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Xaml.Interactivity;

namespace SVNexus.Behaviors;

public class FreezeSizeOnFirstBehavior: Behavior<Control>
{

    public static readonly StyledProperty<bool> FreezeWidthProperty = AvaloniaProperty.Register<FreezeSizeOnFirstBehavior, bool>(
        nameof(FreezeWidth));

    public bool FreezeWidth
    {
        get => GetValue(FreezeWidthProperty);
        set => SetValue(FreezeWidthProperty, value);
    }

    public static readonly StyledProperty<bool> FreezeHeightProperty = AvaloniaProperty.Register<FreezeSizeOnFirstBehavior, bool>(
        nameof(FreezeHeight));

    public bool FreezeHeight
    {
        get => GetValue(FreezeHeightProperty);
        set => SetValue(FreezeHeightProperty, value);
    }
    
    private Control? _control;
    
    protected override void OnAttached()
    {
        base.OnAttached();
        _control = AssociatedObject;
        // _control?.SizeChanged += SizeChanged;
        _control?.LayoutUpdated += OnLayoutUpdated;
    }

    private void OnLayoutUpdated(object? sender, EventArgs e)
    {
        if (sender is not Control control)
            return;

        var bounds = control.Bounds;

        // 避免控件还没拿到有效尺寸时就锁死成 0
        if (bounds.Width <= 0 || bounds.Height <= 0)
            return;

        control.LayoutUpdated -= OnLayoutUpdated;

        var width = bounds.Width;
        var height = bounds.Height;

        // 只设 Width/Height 通常够用；
        // 同时设 Min/Max 可以更强地防止父布局后续改变它。
        if (FreezeWidth)
        {
            control.Width = width;
        }
        
        if (FreezeHeight)
        {
            control.Height = height;
        }

    }

    protected override void OnDetaching()
    {
        base.OnDetaching();
        // _control?.SizeChanged -= SizeChanged;
        _control?.LayoutUpdated -= OnLayoutUpdated;
    }


    // private void SizeChanged(object? sender, SizeChangedEventArgs e)
    // {
    //     if (_control == null)
    //     {
    //         return;
    //     }
    //
    //     if (e.NewSize is not { Width: > 0, Height: > 0 }) return;
    //     
    //     _control.Width = e.NewSize.Width;
    //     _control.Height = e.NewSize.Height;
    //     _control.SizeChanged -= SizeChanged;
    // }
}