using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class Engine
{
    private static readonly Lazy<Engine> _instance =
        new Lazy<Engine>(() => new Engine(), isThreadSafe: true);

    public static Engine Instance => _instance.Value;


    public string Name => "SVNexus";


    public string? DefaultUsername { get; set; }
    
    public string? DefaultPassword { get; set; }


    public CreateContextOptions MakeCreateContextOptions(ContextNotifier notifier)
    {
        return new CreateContextOptions(Name: Name, DefaultUsername: DefaultUsername, DefaultPassword: DefaultPassword, ContextNotifier: notifier);
    }
}