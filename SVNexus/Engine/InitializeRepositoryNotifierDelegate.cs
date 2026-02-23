using System;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class InitializeRepositoryNotifierDelegate: InitializeRepositoryNotifier
{
    public Action? OnCheckoutDirectlyAction { get; init; }
    public Action? OnImportAction { get; init; }
    public Action? OnBackupAction { get; init; }
    public Action<string>? OnBackupFinishedAction { get; init; }
    public Action? OnCheckoutAction { get; init; }
    public Action? OnFinishedAction { get; init; }
    
    public void OnCheckoutDirectly()
    {
        OnCheckoutDirectlyAction?.Invoke();
    }

    public void OnImport()
    {
        OnImportAction?.Invoke();
    }

    public void OnBackup()
    {
        OnBackupAction?.Invoke();
    }

    public void OnBackupFinished(string path)
    {
        OnBackupFinishedAction?.Invoke(path);
    }

    public void OnCheckout()
    {
        OnCheckoutAction?.Invoke();
    }

    public void OnFinished()
    {
        OnFinishedAction?.Invoke();
    }
}