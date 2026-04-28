using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace SVNexus.Components;

public class TargetItem : TemplatedControl
{

    public static readonly StyledProperty<string?> KindIconProperty = AvaloniaProperty.Register<TargetItem, string?>(
        nameof(KindIcon));

    public string? KindIcon
    {
        get => GetValue(KindIconProperty);
        set => SetValue(KindIconProperty, value);
    }

    public static readonly StyledProperty<string?> FileNameProperty = AvaloniaProperty.Register<TargetItem, string?>(
        nameof(FileName));

    public string? FileName
    {
        get => GetValue(FileNameProperty);
        set => SetValue(FileNameProperty, value);
    }

    public static readonly StyledProperty<string?> StatusIconProperty = AvaloniaProperty.Register<TargetItem, string?>(
        nameof(StatusIcon));

    public string? StatusIcon
    {
        get => GetValue(StatusIconProperty);
        set => SetValue(StatusIconProperty, value);
    }

    public static readonly StyledProperty<bool> ShowRelateDirectoryProperty = AvaloniaProperty.Register<TargetItem, bool>(
        nameof(ShowRelateDirectory));

    public bool ShowRelateDirectory
    {
        get => GetValue(ShowRelateDirectoryProperty);
        set => SetValue(ShowRelateDirectoryProperty, value);
    }

    public static readonly StyledProperty<object?> TextToolTipProperty = AvaloniaProperty.Register<TargetItem, object?>(
        nameof(TextToolTip));

    public object? TextToolTip
    {
        get => GetValue(TextToolTipProperty);
        set => SetValue(TextToolTipProperty, value);
    }

    public static readonly StyledProperty<object?> StatusToolTipProperty = AvaloniaProperty.Register<TargetItem, object?>(
        nameof(StatusToolTip));

    public object? StatusToolTip
    {
        get => GetValue(StatusToolTipProperty);
        set => SetValue(StatusToolTipProperty, value);
    }

    public static readonly StyledProperty<string?> RelativeDirectoryProperty = AvaloniaProperty.Register<TargetItem, string?>(
        nameof(RelativeDirectory));

    public string? RelativeDirectory
    {
        get => GetValue(RelativeDirectoryProperty);
        set => SetValue(RelativeDirectoryProperty, value);
    }

    public static readonly StyledProperty<bool> IsDeleteProperty = AvaloniaProperty.Register<TargetItem, bool>(
        nameof(IsDelete));

    public bool IsDelete
    {
        get => GetValue(IsDeleteProperty);
        set => SetValue(IsDeleteProperty, value);
    }

    public static readonly StyledProperty<bool> IsLockedProperty = AvaloniaProperty.Register<TargetItem, bool>(
        nameof(IsLocked), defaultValue: false);

    public bool IsLocked
    {
        get => GetValue(IsLockedProperty);
        set => SetValue(IsLockedProperty, value);
    }

    public static readonly StyledProperty<bool> IsLoadingProperty = AvaloniaProperty.Register<TargetItem, bool>(
        nameof(IsLoading));

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set => SetValue(IsLoadingProperty, value);
    }
    
}