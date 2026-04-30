using System;
using System.Security;
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
    
    public Func<WorkingCopyConflictDescription, WorkingCopyConflictResult> ConflictFunc { get; init; }
    
    
    public string? DialogHostId { get; init; }


    public ContextNotifierDelegate()
    {
        CancelFunc = OnCancel;
        AuthenticateFunc = OnAuthenticate;
        SslServerTrustPromptFunc = OnSslServerTrustPrompt;
        ConflictFunc = OnConflict;
        MaySavePasswordAsPlainTextFunc = OnMaySavePasswordAsPlainText;
    }

    private WorkingCopyConflictResult OnConflict(WorkingCopyConflictDescription description)
    {
        return new WorkingCopyConflictResult(WorkingCopyConflictChoice.Postpone, null, false, null);
    }
    
    private readonly Lock _lock = new();
    
    private string? _cancelMessage;

    public string? CancelMessage
    {
        get
        {
            lock (_lock)
            {
                return _cancelMessage;
            }
        }
        set
        {
            lock (_lock)
            {
                _cancelMessage = value;
            }
        }
    }

    private string? OnCancel()
    {
        lock (_lock)
        {
            if (_cancelMessage == null) return null;
            var msg =  _cancelMessage;
            _cancelMessage = null;
            return msg;

        }
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
            await OverlayDialog.ShowStandardAsync<TrustServerDialog, TrustServerDialogModel>(trustServerDialogModel, options: options, hostId: DialogHostId);
            return new TrustServer(failures, trustServerDialogModel.Save);
        }).Result;
    }


    private Authentication? OnAuthenticate(string realm, string username, bool maySave)
    {
        Console.WriteLine("OnAuthenticate: realm={0}, username={1}, maySave={2}", realm, username, maySave);
        return Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions()
            {
                IsCloseButtonVisible = false,
                Title = "Info",
                Buttons = DialogButton.None
            };
            var authenticateDialogModel = new AuthenticateDialogModel()
            {
                Realm =  realm,
                Username =  username,
                Savable =  maySave,
            };
            await OverlayDialog.ShowStandardAsync<AuthenticateDialog, AuthenticateDialogModel>(authenticateDialogModel, options: options, hostId: DialogHostId);
            return authenticateDialogModel.Accept ? new Authentication(Username: authenticateDialogModel.Username, Password: authenticateDialogModel.Password, Save: authenticateDialogModel.Save) : null;
        }).Result;
        
    }


    private bool OnMaySavePasswordAsPlainText(string realm)
    {
        return Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var result = await OverlayMessageBox.ShowAsync(realm, "Whether to save password as plain text", DialogHostId, MessageBoxIcon.Question, MessageBoxButton.YesNo);

            return result == MessageBoxResult.Yes;
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

    public WorkingCopyConflictResult Conflict(WorkingCopyConflictDescription description)
    {
        throw new NotImplementedException();
    }
}
