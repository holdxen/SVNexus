using System;
using Avalonia.Threading;
using SVNexus.Generated;
using SVNexus.ViewModels;
using SVNexus.Views;
using Ursa.Controls;
using Lock = System.Threading.Lock;

namespace SVNexus.Engine;

public sealed class ContextNotifierDelegate : ContextNotifier
{
    public Func<string, bool>? MaySavePasswordAsPlainTextFunc { get; init; }
    public Action<WorkingCopyNotify>? WorkingCopyNotifyAction { get; init; }
    public Func<string, uint, SslServerCertInfo, bool, TrustServer?>? SslServerTrustPromptFunc { get; init; }
    public Func<string?>? CancelFunc { get; init; }
    public Action<long, long>? ProgressNotifyAction { get; init; }
    public Func<string, string, bool, Authentication?>? AuthenticateFunc { get; init; }
    
    
    public string? DialogHostId { get; init; }


    public ContextNotifierDelegate()
    {
        CancelFunc = OnCancel;
    }
    
    private readonly Lock _lock = new();

    public string? CancelMessage
    {
        get
        {
            lock (_lock)
            {
                return field;
            }
        }
        set
        {
            lock (_lock)
            {
                field = value;
            }
        }
    }

    private string? OnCancel()
    {
        return CancelMessage;
    }
    
    private TrustServer? OnSslServerTrustPrompt(string realm, uint failures, SslServerCertInfo info, bool maySave)
    {
        return Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions()
            {
                IsCloseButtonVisible = false,
                Title = "Info",
                Buttons = DialogButton.None
            };
            var trustServerDialogModel = new TrustServerDialogModel()
            {
                Realm = realm,
                Issuer = info.Issuer,
                AsciiCert = info.AsciiCert,
                Hostname = info.Hostname,
                ValidFrom = info.ValidFrom,
                ValidUntil = info.ValidUntil,
                Fingerprint = info.Fingerprint,
                Savable = maySave,
            };
            await OverlayDialog.ShowModal<TrustServerDialog, TrustServerDialogModel>(trustServerDialogModel, options: options, hostId: DialogHostId);
            return new TrustServer(failures, trustServerDialogModel.Save);
        }).Result;
    }
    

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
