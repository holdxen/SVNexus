using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;

namespace SVNexus.Engine;

public class EngineProxy(IAsyncContext context)
{
    public bool Throw { get; set; }

    public bool Success { get; set; }

    public async Task<MkdirResult?> Mkdir(MkdirOptions options)
    {
        try
        {
            Success = true;
            return await context.Mkdir(options);
        }
        catch (System.Exception e)
        {
            Success = false;
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to create\n{string.Join("\n", options.Paths)}:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
            if (Throw)
            {
                throw;
            }
            return null;
        }
    }

    public async Task Lock(LockOptions options)
    {
        try
        {
            await context.Lock(options);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to lock\n{string.Join("\n", options.Targets)}:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
            if (Throw)
            {
                throw;
            }
        }
    }
    
    public async Task Unlock(UnlockOptions options)
    {
        try
        {
            await context.Unlock(options);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to unlock\n{string.Join("\n", options.Targets)}:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
            if (Throw)
            {
                throw;
            }
        }
    }

    public async Task<uint?[]?> Update(UpdateOptions options)
    {
        try
        {
            return await context.Update(options);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to update\n{string.Join("\n", options.Paths)}:\n{e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
            if (Throw)
            {
                throw;
            }
            return null;
        }
        
    }
}