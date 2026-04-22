using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Metadata;
using Avalonia.Controls.Mixins;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.Media;

namespace SVNexus.Components;

[PseudoClasses(":pressed")]
public class PathIconButton : TemplatedControl
{
    public static readonly StyledProperty<Geometry?> DataProperty = AvaloniaProperty.Register<PathIconButton, Geometry?>(
        nameof(Data));

    public Geometry? Data
    {
        get => GetValue(DataProperty);
        set => SetValue(DataProperty, value);
    }

    public static readonly StyledProperty<PenLineJoin> StrokeJoinProperty = AvaloniaProperty.Register<PathIconButton, PenLineJoin>(
        nameof(StrokeJoin), defaultValue: PenLineJoin.Round);

    public PenLineJoin StrokeJoin
    {
        get => GetValue(StrokeJoinProperty);
        set => SetValue(StrokeJoinProperty, value);
    }

    public static readonly StyledProperty<PenLineCap> StrokeLineCapProperty = AvaloniaProperty.Register<PathIconButton, PenLineCap>(
        nameof(StrokeLineCap),defaultValue: PenLineCap.Round);

    public PenLineCap StrokeLineCap
    {
        get => GetValue(StrokeLineCapProperty);
        set => SetValue(StrokeLineCapProperty, value);
    }

    public static readonly StyledProperty<double> StrokeThicknessProperty = AvaloniaProperty.Register<PathIconButton, double>(
        nameof(StrokeThickness), defaultValue: 2);

    public double StrokeThickness
    {
        get => GetValue(StrokeThicknessProperty);
        set => SetValue(StrokeThicknessProperty, value);
    }


    public static readonly StyledProperty<Stretch> StretchProperty = AvaloniaProperty.Register<PathIconButton, Stretch>(
        nameof(Stretch), defaultValue: Stretch.Uniform);

    public Stretch Stretch
    {
        get => GetValue(StretchProperty);
        set => SetValue(StretchProperty, value);
    }
    
    
    public static readonly StyledProperty<ICommand?> CommandProperty =
        AvaloniaProperty.Register<PathIconButton, ICommand?>(nameof(Command));

    public static readonly StyledProperty<object?> CommandParameterProperty =
        AvaloniaProperty.Register<PathIconButton, object?>(nameof(CommandParameter));

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


    public static readonly StyledProperty<double> SizeProperty = AvaloniaProperty.Register<PathIconButton, double>(
        nameof(Size));

    public double Size
    {
        get => GetValue(SizeProperty);
        set => SetValue(SizeProperty, value);
    }


    public static readonly StyledProperty<Thickness> SpacingProperty = AvaloniaProperty.Register<PathIconButton, Thickness>(
        nameof(Spacing), defaultValue: new  Thickness(2));

    public Thickness Spacing
    {
        get => GetValue(SpacingProperty);
        set => SetValue(SpacingProperty, value);
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