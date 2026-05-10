using System;
using Avalonia.Threading;
using SVNexus.Generated;

namespace SVNexus.Engine;

public class InitializeRepositoryNotifierDelegate: InitializeRepositoryNotifier
{
    
    public bool Dispatch { get; init; }
    
    public Action? OnCheckoutDirectlyAction { get; init; }
    public Action? OnImportAction { get; init; }
    public Action? OnBackupAction { get; init; }
    public Action<string>? OnBackupFinishedAction { get; init; }
    public Action? OnCheckoutAction { get; init; }
    public Action? OnFinishedAction { get; init; }

    public void DispatchOrNot(Action action)
    {
        if (Dispatch)
        {
            Dispatcher.UIThread.Invoke(action);
        }
        else
        {
            action();
        }
    }
    
    public void OnCheckoutDirectly()
    {
        DispatchOrNot(() => OnCheckoutDirectlyAction?.Invoke());
    }

    public void OnImport()
    {
        DispatchOrNot(() => OnImportAction?.Invoke());
    }

    public void OnBackup()
    {
        DispatchOrNot(() => OnBackupAction?.Invoke());
    }

    public void OnBackupFinished(string path)
    {
        DispatchOrNot(() => OnBackupFinishedAction?.Invoke(path));
    }

    public void OnCheckout()
    {
        DispatchOrNot(() => OnCheckoutAction?.Invoke());
    }

    public void OnFinished()
    {
        DispatchOrNot(() => OnFinishedAction?.Invoke());
    }
}