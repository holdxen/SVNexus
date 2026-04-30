using Avalonia;
using System;
using SVNexus.Extension;
using SVNexus.Generated;

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
        DatabaseConnectionExtension.Create().Wait();
        BuildAvaloniaApp()
            .StartWithClassicDesktopLifetime(args);
    }

    // Avalonia configuration, don't remove; also used by visual designer.
    private static AppBuilder BuildAvaloniaApp()
        => AppBuilder.Configure<App>()
            .UsePlatformDetect()
            .WithInterFont()
            .WithDataAnnotationsValidation()
            .LogToTrace();
}