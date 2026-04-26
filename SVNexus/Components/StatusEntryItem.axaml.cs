using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace SVNexus.Components;

public class StatusEntryItem : TemplatedControl
{

    public static readonly StyledProperty<string?> KindIconProperty = AvaloniaProperty.Register<StatusEntryItem, string?>(
        nameof(KindIcon));

    public string? KindIcon
    {
        get => GetValue(KindIconProperty);
        set => SetValue(KindIconProperty, value);
    }

    public static readonly StyledProperty<string?> FileNameProperty = AvaloniaProperty.Register<StatusEntryItem, string?>(
        nameof(FileName));

    public string? FileName
    {
        get => GetValue(FileNameProperty);
        set => SetValue(FileNameProperty, value);
    }

    public static readonly StyledProperty<string?> StatusIconProperty = AvaloniaProperty.Register<StatusEntryItem, string?>(
        nameof(StatusIcon));

    public string? StatusIcon
    {
        get => GetValue(StatusIconProperty);
        set => SetValue(StatusIconProperty, value);
    }

    public static readonly StyledProperty<bool> ShowRelateDirectoryProperty = AvaloniaProperty.Register<StatusEntryItem, bool>(
        nameof(ShowRelateDirectory));

    public bool ShowRelateDirectory
    {
        get => GetValue(ShowRelateDirectoryProperty);
        set => SetValue(ShowRelateDirectoryProperty, value);
    }

    public static readonly StyledProperty<object?> TextToolTipProperty = AvaloniaProperty.Register<StatusEntryItem, object?>(
        nameof(TextToolTip));

    public object? TextToolTip
    {
        get => GetValue(TextToolTipProperty);
        set => SetValue(TextToolTipProperty, value);
    }

    public static readonly StyledProperty<object?> StatusToolTipProperty = AvaloniaProperty.Register<StatusEntryItem, object?>(
        nameof(StatusToolTip));

    public object? StatusToolTip
    {
        get => GetValue(StatusToolTipProperty);
        set => SetValue(StatusToolTipProperty, value);
    }

    public static readonly StyledProperty<string?> RelativeDirectoryProperty = AvaloniaProperty.Register<StatusEntryItem, string?>(
        nameof(RelativeDirectory));

    public string? RelativeDirectory
    {
        get => GetValue(RelativeDirectoryProperty);
        set => SetValue(RelativeDirectoryProperty, value);
    }

    public static readonly StyledProperty<bool> IsDeleteProperty = AvaloniaProperty.Register<StatusEntryItem, bool>(
        nameof(IsDelete));

    public bool IsDelete
    {
        get => GetValue(IsDeleteProperty);
        set => SetValue(IsDeleteProperty, value);
    }

    public static readonly StyledProperty<bool> IsLockedProperty = AvaloniaProperty.Register<StatusEntryItem, bool>(
        nameof(IsLocked), defaultValue: false);

    public bool IsLocked
    {
        get => GetValue(IsLockedProperty);
        set => SetValue(IsLockedProperty, value);
    }

    public static readonly StyledProperty<bool> IsLoadingProperty = AvaloniaProperty.Register<StatusEntryItem, bool>(
        nameof(IsLoading));

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set => SetValue(IsLoadingProperty, value);
    }
    
}