using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace SVNexus.Components;

public class LoadingSvgButton : TemplatedControl
{
    public static readonly StyledProperty<bool> IsLoadingProperty = AvaloniaProperty.Register<LoadingSvgButton, bool>(
        nameof(IsLoading));

    public bool IsLoading
    {
        get => GetValue(IsLoadingProperty);
        set => SetValue(IsLoadingProperty, value);
    }
    
    
        public static readonly StyledProperty<string?> SourceProperty =
        AvaloniaProperty.Register<LoadingSvgButton, string?>(nameof(Source));

    public static readonly StyledProperty<ICommand?> CommandProperty =
        AvaloniaProperty.Register<LoadingSvgButton, ICommand?>(nameof(Command));

    public static readonly StyledProperty<object?> CommandParameterProperty =
        AvaloniaProperty.Register<LoadingSvgButton, object?>(nameof(CommandParameter));

    public static readonly StyledProperty<FlyoutBase?> FlyoutProperty = AvaloniaProperty.Register<LoadingSvgButton, FlyoutBase?>(
        nameof(Flyout));

    public static readonly StyledProperty<bool> EnableFlyoutProperty = AvaloniaProperty.Register<LoadingSvgButton, bool>(
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
    
    
    public static readonly StyledProperty<Thickness> SpacingProperty = AvaloniaProperty.Register<LoadingSvgButton, Thickness>(
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


}