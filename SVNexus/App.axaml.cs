using System;
using System.Collections.Generic;
using System.IO;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core;
using Avalonia.Data.Core.Plugins;
using System.Linq;
using System.Runtime.InteropServices;
using Avalonia.Markup.Xaml;
using Avalonia.Platform;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.ViewModels;
using SVNexus.Views;
using MainWindowViewModel = SVNexus.ViewModels.MainWindowViewModel;

namespace SVNexus;

public partial class App : Application, IRecipient<OnSetThemeVariant>
{
    public App()
    {
        Manager.Default.RegisterAllMessages(this, Manager.AppToken);
    }

    private static byte[][] BuiltinFonts()
    {
        var folderUri = new Uri("avares://SVNexus/Assets/Fonts/");

        // baseUri 对于绝对 uri 可以传 null
        var asserts = new List<byte[]>();
        foreach (var uri in AssetLoader.GetAssets(folderUri, baseUri: null))
        {
            using var stream = AssetLoader.Open(uri);
            using var ms = new MemoryStream();
            stream.CopyTo(ms);
            var bytes = ms.ToArray();
            asserts.Add(bytes);
        }
        return asserts.ToArray();
    }

    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
        EngineMethods.SetupSvg(BuiltinFonts());
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

    public void Receive(OnSetThemeVariant message)
    {
        RequestedThemeVariant = message.Value;
    }
}