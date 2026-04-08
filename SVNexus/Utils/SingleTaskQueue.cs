using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using Avalonia.Threading;

namespace SVNexus.Utils;

public sealed class SingleTaskQueue : IDisposable
{
    private sealed class TaskMessage
    {
        public required CancellationToken CancellationToken { get; init; }
        public required object Task { get; init; }
        public required Action Finished { get; init; }
        
        public TaskCompletionSource<object?>? TaskCompletionSource { get; init; }
    }
    
    private Channel<TaskMessage>? TaskQueue { get; set; }
    
    public event Func<Task?>? QueueEmpty;
    
    private List<CancellationTokenSource> Tokens { get; set; } = [];

    public bool Single { get; set; } = true;
    
    private static void VerifyAccess()
    {
        if (!Dispatcher.UIThread.CheckAccess())
        {
            throw new InvalidOperationException(
                $"当前方法必须在 UI 线程上调用。当前线程: {Environment.CurrentManagedThreadId}");
        }
    }
    

    // private async Task Add(object task, bool cancelOthers = true)
    // {
    //     VerifyAccess();
    //     if (TaskQueue is null)
    //     {
    //         TaskQueue = Channel.CreateUnbounded<TaskMessage>(new UnboundedChannelOptions()
    //         {
    //             SingleReader = true,
    //             SingleWriter = false
    //         });
    //         _ = Dispatcher.UIThread.InvokeAsync(async () =>
    //         {
    //             while (true)
    //             {
    //                 TaskMessage? msg = null;
    //                 try
    //                 {
    //                     // var msg = await TaskQueue.Reader.ReadAsync();
    //                     if (!(TaskQueue?.Reader.TryRead(out msg) ?? false))
    //                     {
    //                         var awaiter = QueueEmpty?.Invoke();
    //                         if (awaiter != null)
    //                         {
    //                             await awaiter;
    //                         }
    //
    //                         TaskQueue = null;
    //                         break;
    //                     }
    //
    //                     switch (msg.Task)
    //                     {
    //                         case Func<CancellationToken, Task> func:
    //                             await func.Invoke(msg.CancellationToken);
    //                             break;
    //                         case Action<CancellationToken> action:
    //                             action.Invoke(msg.CancellationToken);
    //                             break;
    //                     }
    //                 }
    //                 catch (ChannelClosedException)
    //                 {
    //                     break;
    //                 }
    //                 catch (OperationCanceledException)
    //                 {
    //                     Logger.Info("Task is cancelled");
    //                 }
    //                 finally
    //                 {
    //                     msg?.Finished();
    //                 }
    //             }
    //
    //         });
    //     }
    //     var cts = new CancellationTokenSource();
    //     var tokens = Tokens;
    //     Tokens = [cts];
    //     await TaskQueue.Writer.WriteAsync(new TaskMessage
    //     {
    //         CancellationToken = cts.Token,
    //         Task = task,
    //         Finished = () =>
    //         {
    //             Tokens.Remove(cts);
    //         },
    //     }, CancellationToken.None);
    //     if (cancelOthers)
    //     {
    //         foreach (var token in tokens)
    //         {
    //             await token.CancelAsync();
    //         }
    //     }
    // }
    
    
    private async Task Add(object task, TaskCompletionSource<object?>? completionSource = null, bool cancelOthers = true)
    {
        VerifyAccess();
        if (TaskQueue is null)
        {
            TaskQueue = Channel.CreateUnbounded<TaskMessage>(new UnboundedChannelOptions()
            {
                SingleReader = true,
                SingleWriter = false
            });

            _ = Dispatcher.UIThread.InvokeAsync(async () =>
            {
                while (true)
                {
                    TaskMessage? msg = null;
                    try
                    {
                        if (!(TaskQueue?.Reader.TryRead(out msg) ?? false))
                        {
                            var awaiter = QueueEmpty?.Invoke();
                            if (awaiter != null)
                            {
                                await awaiter;
                            }

                            TaskQueue = null;
                            break;
                        }

                        switch (msg.Task)
                        {
                            case Func<CancellationToken, Task> func:
                                await func.Invoke(msg.CancellationToken);
                                break;
                            case Action<CancellationToken> action:
                                action.Invoke(msg.CancellationToken);
                                break;
                        }

                        msg.TaskCompletionSource?.TrySetResult(null);
                    }
                    catch (ChannelClosedException ex)
                    {
                        msg?.TaskCompletionSource?.TrySetException(ex);
                        break;
                    }
                    catch (OperationCanceledException ex)
                    {
                        Logger.Info("Task is cancelled");
                        msg?.TaskCompletionSource?.TrySetCanceled(ex.CancellationToken);
                    }
                    catch (Exception ex)
                    {
                        Logger.Error($"Task execution failed, {ex}");
                        msg?.TaskCompletionSource?.TrySetException(ex);
                    }
                    finally
                    {
                        msg?.Finished();
                    }
                }
            });
        }

        var cts = new CancellationTokenSource();
        var tokens = Tokens;
        Tokens = [cts];

        await TaskQueue.Writer.WriteAsync(new TaskMessage
        {
            CancellationToken = cts.Token,
            Task = task,
            Finished = () =>
            {
                Tokens.Remove(cts);
                cts.Dispose();
            },
            TaskCompletionSource = completionSource
        }, CancellationToken.None);

        if (cancelOthers)
        {
            foreach (var token in tokens)
            {
                await token.CancelAsync();
            }
        }
    }

    public async Task RunAndWait(Action<CancellationToken> task, bool? cancelOthers = null)
    {
        var tcs = new TaskCompletionSource<object?>(
            TaskCreationOptions.RunContinuationsAsynchronously);

        await Add(task, tcs, cancelOthers ?? Single);
        await tcs.Task;
    }
    
    public async Task RunAndWait(Func<CancellationToken, Task> task, bool? cancelOthers = null)
    {
        var tcs = new TaskCompletionSource<object?>(
            TaskCreationOptions.RunContinuationsAsynchronously);

        await Add(task, tcs, cancelOthers ?? Single);
        var r = await tcs.Task;
    }

    public Task Run(Action<CancellationToken> task, bool? cancelOthers = null)
    {
        return Add(task, null, cancelOthers ?? Single);
    }

    public Task Run(Func<CancellationToken, Task> task, bool? cancelOthers = null)
    {
        return Add(task, null, cancelOthers ?? Single);
    }

    private void ReleaseUnmanagedResources()
    {
        TaskQueue?.Writer.Complete();
        TaskQueue = null;
        foreach (var token in Tokens)
        {
            token.Dispose();
        }
        Tokens.Clear();
    }

    public void Dispose()
    {
        ReleaseUnmanagedResources();
        GC.SuppressFinalize(this);
    }

    ~SingleTaskQueue()
    {
        ReleaseUnmanagedResources();
    }
}