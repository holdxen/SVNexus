using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Metadata;

namespace SVNexus.Components;

public class LoadingDot : TemplatedControl
{
    public enum State
    {
        Ready,
        Running,
        Success,
        Failure
    }

    public static readonly StyledProperty<State> StateValueProperty = AvaloniaProperty.Register<LoadingDot, State>(
        nameof(StateValue), defaultValue: State.Ready);

    public State StateValue
    {
        get => GetValue(StateValueProperty);
        set => SetValue(StateValueProperty, value);
    }

    public static readonly StyledProperty<object?> ContentProperty = AvaloniaProperty.Register<LoadingDot, object?>(
        nameof(Content));

    [Content]
    public object? Content
    {
        get => GetValue(ContentProperty);
        set => SetValue(ContentProperty, value);
    }
}