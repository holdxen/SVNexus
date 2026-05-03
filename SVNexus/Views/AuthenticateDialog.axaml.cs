using Avalonia;
using Avalonia.Controls;
using Avalonia.Interactivity;
using Avalonia.Markup.Xaml;

namespace SVNexus.Views;

public partial class AuthenticateDialog : UserControl
{
    public AuthenticateDialog()
    {
        InitializeComponent();
    }

    protected override void OnLoaded(RoutedEventArgs e)
    {
        base.OnLoaded(e);
        UsernameTextBox.Focus();
    }
}