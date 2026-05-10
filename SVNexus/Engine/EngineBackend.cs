using System;
using System.Text.Json;
using System.Text.Json.Serialization;
using System.Text.Json.Serialization.Metadata;
using SVNexus.Generated;
using SVNexus.Utils;

namespace SVNexus.Engine;



public partial class EngineBackend
{
    public static readonly JsonSerializerOptions JsonOptions = new()
    {
        PropertyNameCaseInsensitive = true,
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower,
        TypeInfoResolver = EngineJsonContext.Default
    };


    [JsonSerializable(typeof(SvnError))]
    [JsonSerializable(typeof(ErrorInfo))]
    private partial class EngineJsonContext : JsonSerializerContext;



    public static EngineBackend Instance => Lazy.Value;
    
    private static readonly Lazy<EngineBackend> Lazy =
        new(() => new EngineBackend(), isThreadSafe: true);

    public SingleTaskQueue DatabaseQueue { get; } = new()
    {
        Single = false
    };

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
    
    
    public AsyncContext SimpleContext(ContextNotifier notifier)
    {
        var config = new Config(Proxies: Proxies);
        var opt = new CreateContextOptions(Name: Name, DefaultUsername: DefaultUsername, DefaultPassword: DefaultPassword, ContextNotifier: notifier, Config: config);
        return AsyncContext.Create(opt);
    }
}