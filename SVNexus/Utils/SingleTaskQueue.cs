using System;
using System.Collections.Generic;
using System.Threading;
using System.Threading.Channels;
using System.Threading.Tasks;
using Avalonia.Threading;
using Avalonia.Xaml.Interactions.Custom;
using SVNexus.Generated;

namespace SVNexus.Utils;

public sealed class SingleTaskQueue : IDisposable
{
    private sealed class TaskMessage
    {
        public required CancellationToken CancellationToken { get; init; }
        public required object Task { get; init; }
        public required Action Finished { get; init; }
    }
    
    private Channel<TaskMessage>? TaskQueue { get; set; }
    //     =  Channel.CreateUnbounded<TaskMessage>(new UnboundedChannelOptions()
    // {
    //     SingleReader = true,
    //     SingleWriter = false
    // });
    
    public event Func<Task?>? QueueEmpty;
    
    public List<CancellationTokenSource> Tokens { get; set; } = [];

    // public SingleTaskQueue(bool start = true)
    // {
    //     if (start)
    //     {
    //         Start();
    //     }
    // }

    private async Task Add(object task, bool cancelOthers = true)
    {
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
                        // var msg = await TaskQueue.Reader.ReadAsync();
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

                        switch (msg?.Task)
                        {
                            case Func<CancellationToken, Task> func:
                                await func.Invoke(msg.CancellationToken);
                                break;
                            case Action<CancellationToken> action:
                                action.Invoke(msg.CancellationToken);
                                break;
                        }
                    }
                    catch (ChannelClosedException)
                    {
                        break;
                    }
                    catch (OperationCanceledException)
                    {
                        Logger.Info("Task is cancelled");
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
            },
        }, CancellationToken.None);
        if (cancelOthers)
        {
            foreach (var token in tokens)
            {
                await token.CancelAsync();
            }
        }
        // if (cancelOthers)
        // {
        //     foreach (var token in Tokens)
        //     {
        //         await token.CancelAsync();
        //     }
        //     Tokens.Clear();
        // }
        //
        // var cts = new CancellationTokenSource();
        //
        // Tokens.Add(cts);
        //
        // await TaskQueue.Writer.WriteAsync(new TaskMessage()
        // {
        //     CancellationToken = cts.Token,
        //     Task = task,
        //     Finished = () =>
        //     {
        //         Tokens.Remove(cts);
        //     },
        // }, CancellationToken.None);
    }

    public Task Push(Action<CancellationToken> task, bool cancelOthers = true)
    {
        return Add(task, cancelOthers);
    }

    public Task Run(Func<CancellationToken, Task> task, bool cancelOthers = true)
    {
        return Add(task, cancelOthers);
    }

    // public void Start()
    // {
    //     Dispatcher.UIThread.InvokeAsync(async () =>
    //     {
    //         while (true)
    //         {
    //             TaskMessage? msg = null;
    //             try
    //             {
    //                 // var msg = await TaskQueue.Reader.ReadAsync();
    //                 if (!TaskQueue.Reader.TryRead(out msg))
    //                 {
    //                     var awaiter = QueueEmpty?.Invoke();
    //                     if (awaiter != null)
    //                     {
    //                         await awaiter;
    //                     }
    //
    //                     await TaskQueue.Reader.WaitToReadAsync();
    //                     continue;
    //                 }
    //
    //                 switch (msg.Task)
    //                 {
    //                     case Func<CancellationToken, Task> func:
    //                         await func.Invoke(msg.CancellationToken);
    //                         break;
    //                     case Action<CancellationToken> action:
    //                         action.Invoke(msg.CancellationToken);
    //                         break;
    //                 }
    //             }
    //             catch (ChannelClosedException)
    //             {
    //                 break;
    //             }
    //             catch (OperationCanceledException)
    //             {
    //                 Log.Info("Task is cancelled");
    //             }
    //             finally
    //             {
    //                 msg?.Finished();
    //             }
    //         }
    //     });
    // }

    private void ReleaseUnmanagedResources()
    {
        TaskQueue?.Writer.Complete();
        TaskQueue = null;
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