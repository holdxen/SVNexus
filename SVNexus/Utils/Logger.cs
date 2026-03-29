using System.Runtime.CompilerServices;
using SVNexus.Generated;

namespace SVNexus.Utils;

public static class Logger
{
    public static void Trace(string message,
        [CallerMemberName] string memberName = "",
        [CallerFilePath] string sourceFilePath = "",
        [CallerLineNumber] int sourceLineNumber = 0)
    {
        EngineMethods.LogTrace(sourceLineNumber,sourceFilePath, memberName, message);
    }
    
    public static void Debug(string message,
        [CallerMemberName] string memberName = "",
        [CallerFilePath] string sourceFilePath = "",
        [CallerLineNumber] int sourceLineNumber = 0)
    {
        EngineMethods.LogDebug(sourceLineNumber,sourceFilePath, memberName, message);
    }
    
    public static void Info(string message,
        [CallerMemberName] string memberName = "",
        [CallerFilePath] string sourceFilePath = "",
        [CallerLineNumber] int sourceLineNumber = 0)
    {
        EngineMethods.LogInfo(sourceLineNumber,sourceFilePath, memberName, message);
    }
    
    public static void Warn(string message,
        [CallerMemberName] string memberName = "",
        [CallerFilePath] string sourceFilePath = "",
        [CallerLineNumber] int sourceLineNumber = 0)
    {
        EngineMethods.LogWarn(sourceLineNumber,sourceFilePath, memberName, message);
    }
    
    public static void Error(string message,
        [CallerMemberName] string memberName = "",
        [CallerFilePath] string sourceFilePath = "",
        [CallerLineNumber] int sourceLineNumber = 0)
    {
        EngineMethods.LogError(sourceLineNumber,sourceFilePath, memberName, message);
    }
}