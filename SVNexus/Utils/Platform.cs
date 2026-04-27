using System;
using System.Diagnostics;
using System.IO;
using System.Runtime.InteropServices;
using System.Threading.Tasks;
using SVNexus.Extension;
using SVNexus.Generated;

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
        return Task.CompletedTask;
    }

    public string FileSystemRootPath => "C:\\";
}

public class LinuxPlatform : IPlatform
{
    public Task OpenTerminal(string currentPath)
    {
        var executablePath = $"{AppContext.BaseDirectory}/{IPlatform.SubversionExecutable}";

        string[] candidates =
        [
            "gnome-terminal", "konsole", "xfce4-terminal", "mate-terminal",
            "tilix", "lxterminal", "alacritty", "kitty", "terminator", "xterm"
        ];

        string? terminal = null;

        var x = EngineMethods.FindWhich("x-terminal-emulator");

        if (x is not null)
        {
            var info = new FileInfo(x);
            if (info.LinkTarget is not null)
            {
                var target = info.ResolveLinkTarget(returnFinalTarget: true);
                if (target is not null)
                {
                    terminal = target.FullName;
                }
            }
        }

        if (terminal is null)
        {
            foreach (var candidate in candidates)
            {
                var find = EngineMethods.FindWhich(candidate);
                if (find is null) continue;
                terminal = find;
                break;
            }
        }

        if (terminal is null)
        {
            Logger.Warn("Failed to find default terminal");
            return Task.CompletedTask;
        }
        
        var psi = new ProcessStartInfo
        {
            FileName = terminal,
            WorkingDirectory = currentPath,
            UseShellExecute = false   // 关键：必须 false 才能写 Environment
        };

        var name = Path.GetFileName(terminal);
        switch (name)
        {
            case "gnome-terminal":
            case "xfce4-terminal":
            case "mate-terminal":
            case "tilix":
                psi.ArgumentList.Add($"--working-directory={currentPath}");
                break;
            case "konsole":
                psi.ArgumentList.Add("--workdir");
                psi.ArgumentList.Add(currentPath);
                break;
            case "alacritty":
            case "kitty":
                psi.ArgumentList.Add("--working-directory");
                psi.ArgumentList.Add(currentPath);
                break;
            // xterm / x-terminal-emulator 等没有专门的参数，靠 WorkingDirectory
        }


        psi.Environment["PATH"] = Environment.GetEnvironmentVariable("PATH").Map(e => string.IsNullOrEmpty(e) ? executablePath : $"{currentPath}:{e}");
        
        
        Process.Start(psi);


        return Task.CompletedTask;
    }

    public string FileSystemRootPath { get; } = "/";
}