using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public sealed class ContextNotifierDelegate : ContextNotifier
{
    public Func<string, bool>? MaySavePasswordAsPlainTextFunc { get; init; }
    public Action<WorkingCopyNotify>? WorkingCopyNotifyAction { get; init; }
    public Func<string, uint, SslServerCertInfo, bool, TrustServer?>? SslServerTrustPromptFunc { get; init; }
    public Func<string?>? CancelFunc { get; init; }
    public Action<long, long>? ProgressNotifyAction { get; init; }
    public Func<string, string, bool, Authentication?>? AuthenticateFunc { get; init; }

    public bool MaySavePasswordAsPlainText(string realmString)
        => MaySavePasswordAsPlainTextFunc?.Invoke(realmString) ?? false;

    public void WorkingCopyNotify(WorkingCopyNotify notify)
        => WorkingCopyNotifyAction?.Invoke(notify);

    public TrustServer? SslServerTrustPrompt(string realm, uint failures, SslServerCertInfo info, bool maySave)
        => SslServerTrustPromptFunc?.Invoke(realm, failures, info, maySave);

    public string? Cancel()
        => CancelFunc?.Invoke();

    public void ProgressNotify(long pos, long total)
        => ProgressNotifyAction?.Invoke(pos, total);

    public Authentication? Authenticate(string realm, string username, bool maySave)
        => AuthenticateFunc?.Invoke(realm, username, maySave);
}
