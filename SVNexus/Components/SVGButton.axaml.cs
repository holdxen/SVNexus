using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Metadata;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Media;

namespace SVNexus.Components;


[PseudoClasses(":pressed")]
public class SvgButton : TemplatedControl
{
    public static readonly StyledProperty<string?> SourceProperty =
        AvaloniaProperty.Register<SvgButton, string?>(nameof(Source));

    public static readonly StyledProperty<ICommand?> CommandProperty =
        AvaloniaProperty.Register<SvgButton, ICommand?>(nameof(Command));

    public static readonly StyledProperty<object?> CommandParameterProperty =
        AvaloniaProperty.Register<SvgButton, object?>(nameof(CommandParameter));

    public static readonly StyledProperty<FlyoutBase?> FlyoutProperty = AvaloniaProperty.Register<SvgButton, FlyoutBase?>(
        nameof(Flyout));

    public static readonly StyledProperty<bool> EnableFlyoutProperty = AvaloniaProperty.Register<SvgButton, bool>(
        nameof(EnableFlyout));

    public bool EnableFlyout
    {
        get => GetValue(EnableFlyoutProperty);
        set => SetValue(EnableFlyoutProperty, value);
    }

    public FlyoutBase? Flyout
    {
        get => GetValue(FlyoutProperty);
        set => SetValue(FlyoutProperty, value);
    }
    
    public string? Source
    {
        get => GetValue(SourceProperty);
        set => SetValue(SourceProperty, value);
    }

    public ICommand? Command
    {
        get => GetValue(CommandProperty);
        set => SetValue(CommandProperty, value);
    }

    public object? CommandParameter
    {
        get => GetValue(CommandParameterProperty);
        set => SetValue(CommandParameterProperty, value);
    }
    
    
    public static readonly StyledProperty<Thickness> SpacingProperty = AvaloniaProperty.Register<PathIconButton, Thickness>(
        nameof(Spacing), defaultValue: new  Thickness(2));

    public Thickness Spacing
    {
        get => GetValue(SpacingProperty);
        set => SetValue(SpacingProperty, value);
    }
    
    public static readonly StyledProperty<double> SizeProperty =
        AvaloniaProperty.Register<SvgRender, double>(nameof(Size), 24);
    
    public double Size
    {
        get => GetValue(SizeProperty);
        set => SetValue(SizeProperty, value);
    }

    
    protected override void OnPointerPressed(PointerPressedEventArgs e)
    {
        base.OnPointerPressed(e);
        PseudoClasses.Set(":pressed", true);
    }

    protected override void OnPointerReleased(PointerReleasedEventArgs e)
    {
        base.OnPointerReleased(e);
        PseudoClasses.Set(":pressed", false);
    }

    protected override void OnPointerCaptureLost(PointerCaptureLostEventArgs e)
    {
        base.OnPointerCaptureLost(e);
        PseudoClasses.Set(":pressed", false);
    }
}