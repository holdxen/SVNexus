using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Metadata;
using Avalonia.Interactivity;

namespace SVNexus.Components;

public class AdvancedTreeViewItem: TreeViewItem
{
    public static readonly StyledProperty<bool> HasChildProperty = AvaloniaProperty.Register<AdvancedTreeViewItem, bool>(
        nameof(HasChild));

    public bool HasChild
    {
        get => GetValue(HasChildProperty);
        set => SetValue(HasChildProperty, value);
    }

    public static readonly StyledProperty<bool> IsLoadingProperty = AvaloniaProperty.Register<AdvancedTreeViewItem, bool>(
        nameof(IsLoading));

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set => SetValue(IsLoadingProperty, value);
    }
}