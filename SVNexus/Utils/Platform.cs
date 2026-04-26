using System;
using System.Diagnostics;
using System.Runtime.InteropServices;
using System.Threading.Tasks;

namespace SVNexus.Utils;

public interface IPlatform
{
    public Task OpenTerminal(string currentPath);
    
    public string FileSystemRootPath { get; }

    public static IPlatform Create()
    {
        if (RuntimeInformation.IsOSPlatform(OSPlatform.Windows))
        {
            return new WindowsPlatform();
        }

        if (RuntimeInformation.IsOSPlatform(OSPlatform.Linux))
        {
            return new LinuxPlatform();
        }

        return RuntimeInformation.IsOSPlatform(OSPlatform.OSX) ? new MacOsPlatform() : throw new System.NotSupportedException();
    }

    protected static string SubversionExecutable => "svn/bin";
}

public class MacOsPlatform : IPlatform
{
    public Task OpenTerminal(string currentPath)
    {
        // var path = Environment.GetEnvironmentVariable("PATH");
        // path = string.IsNullOrEmpty(path) ? $"{AppContext.BaseDirectory}/{IPlatform.SubversionExecutable}" : $"{AppContext.BaseDirectory}/{IPlatform.SubversionExecutable}:{path}";
        // using var process = new Process();
        // process.StartInfo = new ProcessStartInfo
        // {
        //     FileName = "open",
        //     Arguments = $"-a Terminal \"{currentPath}\"",
        //     UseShellExecute = false,
        //     Environment =
        //     {
        //         {
        //             "PATH",
        //             path
        //         }
        //     }
        // };
        // process.Start();
        
        var executablePath    = $"{AppContext.BaseDirectory}/{IPlatform.SubversionExecutable}";

        var shellCmd = $"cd \\\"{currentPath}\\\" && export PATH=\\\"{executablePath}:$PATH\\\" && printf \\\"\\\\033[2J\\\\033[3J\\\\033[1;1H\\\"";
        

        // 拼出 AppleScript: tell application "Terminal" to do script "..."
        var appleScript = $"tell application \"Terminal\" to do script \"{shellCmd}\"";
        
        Logger.Info($"Script: {appleScript}");

        var psi = new ProcessStartInfo
        {
            FileName = "osascript",
            UseShellExecute = false
        };
        psi.ArgumentList.Add("-e");
        psi.ArgumentList.Add(appleScript);
        psi.ArgumentList.Add("-e");
        psi.ArgumentList.Add("tell application \"Terminal\" to activate"); // 把窗口提到前台

        Process.Start(psi);
        
        return Task.CompletedTask;
    }

    public string FileSystemRootPath => "/";
}

public class WindowsPlatform : IPlatform
{
    public Task OpenTerminal(string currentPath)
    {
        throw new System.NotImplementedException();
    }

    public string FileSystemRootPath => "C:\\";
}

public class LinuxPlatform : IPlatform
{
    public Task OpenTerminal(string currentPath)
    {
        throw new System.NotImplementedException();
    }

    public string FileSystemRootPath { get; } = "/";
}