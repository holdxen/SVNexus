using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Net.Http.Headers;
using System.Text;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using Exception = System.Exception;
using Notification = Ursa.Controls.Notification;

namespace SVNexus.ViewModels.WorkingCopy.Remote;

public partial class RemoteSnapshotViewModel: ViewModelBase
{
    
    public partial class FileItemViewModel: ViewModelLite
    {
        [ObservableProperty]
        public partial string Name { get; set; } = string.Empty;
        
        
        [ObservableProperty]
        public partial ListEntry? Entry { get; set; }
        
        public string Path => Entry?.Path ?? string.Empty;
        
        public NodeKind NodeKind => Entry?.Kind ?? NodeKind.File;
        
        public ObservableCollection<FileItemViewModel> Children { get; set; } = [];


        public string NodeKindIcon => NodeKind.NodeKindIcon();

    }
    
    // [ObservableProperty]
    // public partial bool IsEditorLoading { get; set; }

    [ObservableProperty] public partial LoadingOrErrorState EditorState { get; set; } = new LoadingOrErrorState.None();
    
    // [ObservableProperty]
    // public partial bool IsTreeViewLoading { get; set; }

    [ObservableProperty]
    public partial LoadingOrErrorState TreeViewState { get; set; } = new LoadingOrErrorState.None();
    
    public required WeakReferenceMessenger Messenger { get; set; }
    
    public required string Url { get; set; }
    
    public required Revision Revision { get; set; }
    
    
    public ObservableCollection<FileItemViewModel> Files { get; set; } = [];

    [ObservableProperty] public partial string Code { get; set; } = string.Empty;

    [ObservableProperty]
    public partial FileItemViewModel? SelectedItem { get; set; }


    private readonly Dictionary<string, string> _cache = [];

    private void AddCache(string path, string code)
    {
        const int max = 20;

        if (_cache.Count >= max)
        {
            _cache.Remove(_cache.First().Key);
        }

        _cache[path] = code;
    }

    [RelayCommand]
    private async Task TryCatFile(string path)
    {
        // TODO 已经在申请内容的文件不要重复申请
        if (_cache.TryGetValue(path, out var code))
        {
            Code = code;
            EditorState = new LoadingOrErrorState.None();
            return;
        }
        EditorState = LoadingOrErrorState.MakeLoading();
            
        var catOptions = new CatOptions(
            Path: Url + "/" + path,
            PegRevision: new Revision.Head(),
            Revision: Revision,
            ExpandKeywords: false
        );
            
        var hostId = Messenger.Send(new OnGetDialogHostId()).Response;

        using var context = Engine.Engine.Instance.SimpleContext(hostId);
        try
        {
            var result = await context.Cat(catOptions);
            var text = Encoding.UTF8.GetString(result.Content);
            Console.WriteLine($"Cat Text: {text}");
            if (SelectedItem?.Path == path)
            {
                Code = text;
            }
            AddCache(path, text);
        }
        catch (Exception e)
        {
            Console.WriteLine(e);
            if (SelectedItem?.Path == path)
            {
                EditorState = new LoadingOrErrorState.Error()
                {
                    ErrorMessage = e.HumanReadableMessage,
                    RetryCommand = TryCatFileCommand,
                    RetryCommandParameter = path
                };
            }
            throw;
        }
        finally
        {
            if (SelectedItem?.Path == path)
            {
                EditorState = LoadingOrErrorState.MakeNone();
            }
        }
    }

    partial void OnSelectedItemChanged(FileItemViewModel? value)
    {
        if (value is null || value.NodeKind != NodeKind.File)
        {
            Code = string.Empty;
        }
        else
        {
            // var path = Url + "/" + value.Path;
            var path = value.Path;
            Dispatcher.UIThread.InvokeAsync(async () => { await TryCatFile(path); });
            // if (_cache.TryGetValue(path, out var code))
            // {
            //     Code = code;
            //     return;
            // }
            //
            // EditorState = LoadingOrErrorState.MakeLoading();
            //
            // var catOptions = new CatOptions(
            //         Path: Url + "/" + path,
            //         PegRevision: new Revision.Head(),
            //         Revision: Revision,
            //         ExpandKeywords: false
            //     );
            //
            // var hostId = Messenger.Send(new OnGetDialogHostId()).Response;
            //
            // Dispatcher.UIThread.InvokeAsync(async () =>
            // {
            //     using var context = Engine.Engine.Instance.SimpleContext(hostId);
            //     try
            //     {
            //         var result = await context.Cat(catOptions);
            //         var text = Encoding.UTF8.GetString(result.Content);
            //         Console.WriteLine($"Cat Text: {text}");
            //         if (SelectedItem?.Path == path)
            //         {
            //             Code = text;
            //         }
            //         AddCache(path, text);
            //     }
            //     catch (Exception e)
            //     {
            //         Console.WriteLine(e);
            //         if (SelectedItem?.Path == path)
            //         {
            //             EditorState = new LoadingOrErrorState.Error()
            //             {
            //                 ErrorMessage = e.HumanReadableMessage,
            //             };
            //         }
            //         throw;
            //     }
            //     finally
            //     {
            //         if (SelectedItem?.Path == path)
            //         {
            //             EditorState = LoadingOrErrorState.MakeNone();
            //         }
            //     }
            // });
            //


        }
    }
    
    private void HandleException(Exception e)
    {
        Manager.MainWindow.Send(new OnNotification(new Notification
        {
            Title = "Error",
            Content = $"Failed to query: {e.HumanReadableMessage}",
            Type = NotificationType.Error,
        }));
    }

    public void BuildFileTree(ListEntry[] entries)
    {
        Files.Clear();

        foreach (var entry in entries)
        {
            if (entry.Path == "/" || entry.Path.IsWhiteSpace())
            {
                continue;
            }
            var parts = entry.Path.Split('/');
            var index = 0;
            var parent = Files;
            foreach (var part in parts)
            {
                var found = parent.FirstOrDefault(item => item.Name == part);

                if (found is null)
                {
                    var item = new FileItemViewModel()
                    {
                        Name = part,
                    };
                    Console.WriteLine("Add File: {0}", item.Name);
                    if (item.Name.IsWhiteSpace())
                    {
                        Console.WriteLine("Empty File: {0}", entry);
                    }
                    parent.Add(item);
                    if (index == parts.Length - 1)
                    {
                        item.Entry = entry;
                    }
                    parent = item.Children;
                }
                else
                {
                    parent = found.Children;
                    if (index == parts.Length - 1)
                    {
                        found.Entry = entry;
                    }
                }

                index++;
            }
        }
    }
    
    [RelayCommand]
    private async Task OnLoaded()
    {
        var hostId = Messenger.Send(new OnGetDialogHostId()).Response;
        
        using var context = Engine.Engine.Instance.SimpleContext(hostId);

        TreeViewState = LoadingOrErrorState.MakeLoading();

        try
        {
            Console.WriteLine("List: url={0}", Url);
            var listOptions = new ListOptions(
                Path: Url, 
                PegRevision: new Revision.Head(), 
                Revision: Revision, 
                Patterns: null, 
                Depth: Depth.Infinity, 
                DirentCreatedRevision: false, 
                DirentHasProperties: false, 
                DirentKind: true, 
                DirentLastAuthor: true, 
                DirentSize: true, 
                DirentTime: true, 
                FetchLocks: true, 
                IncludeExternals: false);
            
            var result = await context.List(listOptions);
            Console.WriteLine($"List: {result.Entries.Length}");
            BuildFileTree(result.Entries);
        }
        catch (Exception e)
        {
            HandleException(e);
            TreeViewState = new LoadingOrErrorState.Error()
            {
                ErrorMessage = e.HumanReadableMessage,
                RetryCommand = loadedCommand,
                RetryCommandParameter = null
            };
        }
        finally 
        {
            TreeViewState = LoadingOrErrorState.MakeNone();
        }

    }
}