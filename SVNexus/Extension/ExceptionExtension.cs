using System;
using System.Linq;
using System.Text.Json;
using System.Threading.Tasks;
using SVNexus.Generated;
using Exception = SVNexus.Generated.Exception;

namespace SVNexus.Extension;

public static class ExceptionExtension
{
    private static readonly JsonSerializerOptions Options = new()
    {
        PropertyNameCaseInsensitive = true,
        PropertyNamingPolicy = JsonNamingPolicy.SnakeCaseLower
    };

    public static readonly SvnErrnoConstants SvnErrnoConstants = new();
    
    extension(System.Exception e)
    {
        public string HumanReadableMessage
        {
            get
            {
                switch (e)
                {
                    case Exception.SvnException svnException:
                    {
                        var error = JsonSerializer.Deserialize<SvnError>(svnException.Message, Options)!;
                        return string.Join("\n", error.Info.Select(i => i.Msg));
                    }
                    case Exception.AprException aprException:
                    {
                        var error = JsonSerializer.Deserialize<AprError>(aprException.Message, Options)!;
                        return error.Msg;
                    }
                    default:
                        return e.Message;
                }
            }
        }

        public bool Handle(Func<SvnError, bool>? svnExceptionHandler = null, Func<AprError, bool>? aprExceptionHandler = null, Func<System.Exception, bool>? exceptionHandler  = null)
        {
            switch (e)
            {
                case Exception.SvnException svnException when svnExceptionHandler is not null:
                {
                    var error = JsonSerializer.Deserialize<SvnError>(svnException.Message, Options)!;
                    return svnExceptionHandler.Invoke(error);
                }
                case Exception.AprException aprException when aprExceptionHandler is not null:
                {
                    var error = JsonSerializer.Deserialize<AprError>(aprException.Message, Options)!;
                    return aprExceptionHandler.Invoke(error);
                }
                default:
                    return exceptionHandler?.Invoke(e) ?? false;
            }
        }
        
        public async Task<bool> HandleAsync(Func<SvnError, Task<bool>>? svnExceptionHandler = null, Func<AprError, Task<bool>>? aprExceptionHandler = null, Func<System.Exception, Task<bool>>? exceptionHandler  = null)
        {
            switch (e)
            {
                case Exception.SvnException svnException when svnExceptionHandler is not null:
                {
                    var error = JsonSerializer.Deserialize<SvnError>(svnException.Message, Options)!;
                    return await svnExceptionHandler.Invoke(error);
                }
                case Exception.AprException aprException when aprExceptionHandler is not null:
                {
                    var error = JsonSerializer.Deserialize<AprError>(aprException.Message, Options)!;
                    return await aprExceptionHandler.Invoke(error);
                }
                default:
                    var task = exceptionHandler?.Invoke(e);
                    if (task is not null)
                    {
                        return await task;
                    }
                    return false;
                    // return (await exceptionHandler?.Invoke(e)) ?? false;
            }
        }
    }
}