using System;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core;
using Avalonia.Data.Core.Plugins;
using System.Linq;
using System.Runtime.InteropServices;
using Avalonia.Markup.Xaml;
using SVNexus.Generated;
using SVNexus.ViewModels;
using SVNexus.Views;
using MainWindowViewModel = SVNexus.ViewModels.MainWindowViewModel;

namespace SVNexus;

public partial class App : Application
{
    // public static IServiceProvider Services { get; private set; }
    //
    // static App()
    // {
    //     Services = Configure();
    // }
    //
    // private static ServiceProvider Configure()
    // {
    //     var services = new ServiceCollection();
    //
    //     services.AddSingleton<IAppState, AppState>();
    //     
    //     services.AddTransient<MainWindowViewModel>();
    //
    //     return services.BuildServiceProvider();
    // }

    public override void Initialize()
    {
        // Engine.Engine.Instance.Proxies = new Proxies(new Proxy("127.0.0.1", 7890, null, null), null, null);
        AvaloniaXamlLoader.Load(this);
        EngineMethods.SetupSvg(Program.BuiltinFonts());
    }

    public override void OnFrameworkInitializationCompleted()
    {
        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
        {
            // Avoid duplicate validations from both Avalonia and the CommunityToolkit. 
            // More info: https://docs.avaloniaui.net/docs/guides/development-guides/data-validation#manage-validationplugins
            DisableAvaloniaDataAnnotationValidation();
            desktop.MainWindow = new MainWindow
            {
                // DataContext = Services.GetRequiredService<MainWindowViewModel>(),
                DataContext = new MainWindowViewModel(),
            };
        }

        base.OnFrameworkInitializationCompleted();
    }

    private void DisableAvaloniaDataAnnotationValidation()
    {
        // Get an array of plugins to remove
        var dataValidationPluginsToRemove =
            BindingPlugins.DataValidators.OfType<DataAnnotationsValidationPlugin>().ToArray();

        // remove each entry found
        foreach (var plugin in dataValidationPluginsToRemove)
        {
            BindingPlugins.DataValidators.Remove(plugin);
        }
    }
}