using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace SVNexus.Components;

public class GroupBox : ContentControl
{
    // 标题属性
    public static readonly StyledProperty<string> TitleProperty =
        AvaloniaProperty.Register<GroupBox, string>(nameof(Title), "Group");

    public string Title
    {
        get => GetValue(TitleProperty);
        set => SetValue(TitleProperty, value);
    }

    // 是否显示复选框
    public static readonly StyledProperty<bool> IsCheckableProperty =
        AvaloniaProperty.Register<GroupBox, bool>(nameof(IsCheckable), false);

    public bool IsCheckable
    {
        get => GetValue(IsCheckableProperty);
        set => SetValue(IsCheckableProperty, value);
    }

    // 复选框状态
    public static readonly StyledProperty<bool> IsCheckedProperty =
        AvaloniaProperty.Register<GroupBox, bool>(nameof(IsChecked), true);

    public bool IsChecked
    {
        get => GetValue(IsCheckedProperty);
        set => SetValue(IsCheckedProperty, value);
    }
}