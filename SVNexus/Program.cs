using Avalonia;
using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using Avalonia.Dialogs;
using Avalonia.Platform;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Utils;

namespace SVNexus;

internal static class Program
{
    // Initialization code. Don't use any Avalonia, third-party APIs or any
    // SynchronizationContext-reliant code before AppMain is called: things aren't initialized
    // yet and stuff might break.

    private static byte[][]? _fonts;

    public static byte[][] BuiltinFonts()
    {
        if (_fonts is not null)
        {
            return _fonts;
        }
        
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
        _fonts = asserts.ToArray();
        return _fonts;
    }
    
    [STAThread]
    public static void Main(string[] args)
    {
        EngineMethods.EngineInitialize();
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