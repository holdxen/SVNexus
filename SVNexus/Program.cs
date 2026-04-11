using Avalonia;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Dialogs;
using Avalonia.Platform;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Utils;

namespace SVNexus;

internal static class Program
{
    // Initialization code. Don't use any Avalonia, third-party APIs or any
    // SynchronizationContext-reliant code before AppMain is called: things aren't initialized
    // yet and stuff might break.


    
    [STAThread]
    public static void Main(string[] args)
    {
        EngineMethods.EngineInitialize();
        DatabaseManagerExtension.Create().Wait();
        BuildAvaloniaApp()
            .StartWithClassicDesktopLifetime(args);
    }

    // Avalonia configuration, don't remove; also used by visual designer.
    private static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .LogToTrace();
}