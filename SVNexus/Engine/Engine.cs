using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class Engine
{
    private static readonly Lazy<Engine> _instance = new(() => new Engine(), isThreadSafe: true);

    public static Engine Instance => _instance.Value;


    public string Name => "SVNexus";


    public string? DefaultUsername { get; set; }
    
    public string? DefaultPassword { get; set; }


    public Proxies Proxies { get; set; } = new(null, null, null);
    


    public CreateContextOptions MakeCreateContextOptions(ContextNotifier notifier)
    {
        var config = new Config(Proxies: Proxies);
        return new CreateContextOptions(Name: Name, DefaultUsername: DefaultUsername, DefaultPassword: DefaultPassword, ContextNotifier: notifier, Config: config);
    }
}