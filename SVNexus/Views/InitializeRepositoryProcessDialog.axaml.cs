using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml;

namespace SVNexus.Views;

public partial class InitializeRepositoryProcessDialog : UserControl
{

    static InitializeRepositoryProcessDialog()
    {
        ScrollToProperty.Changed.AddClassHandler<InitializeRepositoryProcessDialog, int>(OnScrollToPropertyChanged);
    }

    private static void OnScrollToPropertyChanged(InitializeRepositoryProcessDialog target, AvaloniaPropertyChangedEventArgs<int> args)
    {
        if (args.NewValue.Value >= 0)
        {
            target.LogList.ScrollIntoView(args.NewValue.Value);   
        }
    }

    public InitializeRepositoryProcessDialog()
    {
        InitializeComponent();
    }

    public static readonly StyledProperty<int> ScrollToProperty = AvaloniaProperty.Register<InitializeRepositoryProcessDialog, int>(
        nameof(ScrollTo), defaultValue: 0);

    public int ScrollTo
    {
        get => GetValue(ScrollToProperty);
        set => SetValue(ScrollToProperty, value);
    }
}