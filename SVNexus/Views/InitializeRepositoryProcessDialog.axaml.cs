using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml;

namespace SVNexus.Views;

public partial class InitializeRepositoryProcessDialog : UserControl
{
    public InitializeRepositoryProcessDialog()
    {
        InitializeComponent();
    }

    protected override void OnLoaded(RoutedEventArgs e)
    {
        base.OnLoaded(e);
        
        LogList.ScrollIntoView(0);
    }
}