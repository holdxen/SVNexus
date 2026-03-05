using System;
using System.Linq;
using System.Text.Json;
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

        public void Handle(Action<SvnError>? svnExceptionHandler = null, Action<AprError>? aprExceptionHandler = null, Action<System.Exception>? exceptionHandler  = null)
        {
            switch (e)
            {
                case Exception.SvnException svnException when svnExceptionHandler is not null:
                {
                    var error = JsonSerializer.Deserialize<SvnError>(svnException.Message, Options)!;
                    svnExceptionHandler.Invoke(error);
                    break;
                }
                case Exception.AprException aprException when aprExceptionHandler is not null:
                {
                    var error = JsonSerializer.Deserialize<AprError>(aprException.Message, Options)!;
                    aprExceptionHandler.Invoke(error);
                    break;
                }
                default:
                    exceptionHandler?.Invoke(e);
                    break;
            }
        }
    }
}