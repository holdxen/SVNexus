using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;

namespace SVNexus.Components;

using Lucide.Avalonia;

public class LucideIconButton : TemplatedControl
{
    public static readonly StyledProperty<LucideIconKind> LucideKindProperty = AvaloniaProperty.Register<LucideIconButton, LucideIconKind>(
        nameof(LucideKind));

    public LucideIconKind LucideKind
    {
        get => GetValue(LucideKindProperty);
        set => SetValue(LucideKindProperty, value);
    }


    public static readonly StyledProperty<double> SizeProperty = AvaloniaProperty.Register<LucideIconButton, double>(
        nameof(Size), defaultValue: 12);

    public double Size
    {
        get => GetValue(SizeProperty);
        set => SetValue(SizeProperty, value);
    }
    
    
    public static readonly StyledProperty<ICommand?> CommandProperty =
        AvaloniaProperty.Register<LucideIconButton, ICommand?>(nameof(Command));

    public static readonly StyledProperty<object?> CommandParameterProperty =
        AvaloniaProperty.Register<LucideIconButton, object?>(nameof(CommandParameter));

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


    public static readonly StyledProperty<Thickness> SpacingProperty = AvaloniaProperty.Register<LucideIconButton, Thickness>(
        nameof(Spacing), defaultValue: new Thickness(2));

    public Thickness Spacing
    {
        get => GetValue(SpacingProperty);
        set => SetValue(SpacingProperty, value);
    }


    
}