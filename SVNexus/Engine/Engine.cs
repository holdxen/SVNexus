using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class Engine
{
    public static Engine Instance => Lazy.Value;
    
    private static readonly Lazy<Engine> Lazy =
        new(() => new Engine(), isThreadSafe: true);

    public string Name => "SVNexus";

    public string? DefaultUsername { get; set; }
    
    public string? DefaultPassword { get; set; }


    public Proxies Proxies { get; set; } = new(null, null, null);
    


    public CreateContextOptions MakeCreateContextOptions(ContextNotifier notifier)
    {
        var config = new Config(Proxies: Proxies);
        return new CreateContextOptions(Name: Name, DefaultUsername: DefaultUsername, DefaultPassword: DefaultPassword, ContextNotifier: notifier, Config: config);
    }

    public CreateContextOptions SimpleCreateContextOptions(string? dialogHostId = null)
    {
        var config = new Config(Proxies: Proxies);
        return new CreateContextOptions(Name: Name, DefaultUsername: DefaultUsername, DefaultPassword: DefaultPassword, ContextNotifier: new ContextNotifierDelegate()
        {
            DialogHostId =  dialogHostId,
        }, Config: config);
    }

    public AsyncContext SimpleContext(string? dialogHostId = null)
    {
        return AsyncContext.Create(SimpleCreateContextOptions(dialogHostId));
    }
}