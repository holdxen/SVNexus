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
using SVNexus.Utils;
using Exception = System.Exception;
using Notification = Ursa.Controls.Notification;

namespace SVNexus.ViewModels.WorkingCopy.Remote;

// TODO: allow see inherit property
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

    public class FileCache
    {
        public required ListEntry Entry { get; set; }
        
        public bool IsTextFile { get; set; }
        
        public string Content { get; set; } = string.Empty;
        
        public Encoding? Encoding { get; set; }
        
        public Dictionary<string, string> Properties { get; set; } = [];
    }

    public class PropertyItemViewModel
    {
        public required string Name { get; set; }
        
        public required string Value { get; set; }

    }
    
    
    [ObservableProperty]
    public partial List<PropertyItemViewModel> Properties { get; set; } = [];

    [ObservableProperty]
    public partial bool IsTextView { get; set; } = true;
    

    [ObservableProperty] public partial bool IsText { get; set; } = true;

    [ObservableProperty] public partial LoadingOrErrorState EditorState { get; set; } = new LoadingOrErrorState.None();

    [ObservableProperty]
    public partial LoadingOrErrorState TreeViewState { get; set; } = new LoadingOrErrorState.None();
    
    
    public required string Url { get; set; }
    
    public required Revision Revision { get; set; }
    
    
    public ObservableCollection<FileItemViewModel> Files { get; set; } = [];

    [ObservableProperty] public partial string Code { get; set; } = string.Empty;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(SelectedFileName))]
    [NotifyPropertyChangedFor(nameof(SelectedFileSizeText))]
    [NotifyPropertyChangedFor(nameof(SelectedItemLastAuthor))]
    [NotifyPropertyChangedFor(nameof(SelectedItemHasProperties))]
    public partial FileItemViewModel? SelectedItem { get; set; }


    public string? SelectedFileName => SelectedItem?.Name;

    public string? SelectedFileSizeText => SelectedItem?.Entry?.Size is null
        ? null
        : new FormatSizeOptions(SelectedItem.Entry.Size ?? 0).Format();
    
    [ObservableProperty]
    public partial string? SelectedFileEncodingName { get; set; }
    
    public string? SelectedItemLastAuthor => SelectedItem?.Entry?.LastAuthor;

    public bool SelectedItemHasProperties => SelectedItem?.Entry?.HasProperties ?? false;


    private readonly LimitedDictionary<string, FileCache?> _cache = new()
    {
        Limit = 20
    };

    // private void AddCache(string path, FileCache cache)
    // {
    //     const int max = 20;
    //
    //     if (_cache.Count >= max)
    //     {
    //         _cache.Remove(_cache.First().Key);
    //     }
    //
    //     _cache[path] = cache;
    // }

    [RelayCommand]
    private void OnSelectTextView()
    {
        IsTextView = true;
    }

    [RelayCommand]
    private void OnSelectPropertyView()
    {
        IsTextView = false;
    }


    private void SetCache(FileCache fileCache)
    {
        IsText = fileCache.IsTextFile;
        Properties = fileCache.Properties.Select(e => new PropertyItemViewModel()
        {
            Name = e.Key,
            Value = e.Value
        }).ToList();
        if (fileCache.IsTextFile)
        {
            Code = fileCache.Content;
        }

        SelectedFileEncodingName = fileCache.Encoding?.EncodingName;
        
        
    }

    [RelayCommand]
    private async Task TryCatFile(ListEntry entry)
    {
        if (_cache.TryGetValue(entry.Path, out var cache))
        {
            if (cache is null)
            {
                return;
            }
            SetCache(cache);
            EditorState = new LoadingOrErrorState.None();
            return;
        }
        EditorState = LoadingOrErrorState.MakeLoading();
        _cache[entry.Path] = null;
            
        var catOptions = new CatOptions(
            Path: Url + "/" + entry.Path,
            PegRevision: new Revision.Head(),
            Revision: Revision,
            ExpandKeywords: false
        );
            
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

        using var context = Engine.Engine.Instance.SimpleContext(hostId);
        try
        {
            var result = await context.Cat(catOptions);

            var fileCache = new FileCache
            {
                Entry = entry,
                Properties = result.Properties
            };
            
            if (TextDetector.IsText(result.Content, out var encoding))
            {
                fileCache.IsTextFile = true;
                fileCache.Content = encoding.GetString(result.Content);
                fileCache.Encoding = encoding;
            }
            else
            {
                fileCache.IsTextFile = false;
            }
            

            if (SelectedItem?.Path == entry.Path)
            {
                SetCache(fileCache);
            }
            


            // AddCache(entry.Path, fileCache);
            _cache.Add(entry.Path, fileCache);
        }
        catch (Exception e)
        {
            if (SelectedItem?.Path == entry.Path)
            {
                EditorState = new LoadingOrErrorState.Error()
                {
                    ErrorMessage = e.HumanReadableMessage,
                    RetryCommand = TryCatFileCommand,
                    RetryCommandParameter = entry
                };
            }
        }
        finally
        {
            if (SelectedItem?.Path == entry.Path && EditorState is LoadingOrErrorState.Loading)
            {
                EditorState = LoadingOrErrorState.MakeNone();
            }

            if (_cache.TryGetValue(entry.Path, out var value))
            {
                if (value is null)
                {
                    _cache.Remove(entry.Path);
                }
            }
        }
    }

    partial void OnSelectedItemChanged(FileItemViewModel? value)
    {
        if (value is null)
        {
            Code = string.Empty;
        }
        else if (value.NodeKind is NodeKind.Directory)
        {
            Code = string.Empty;
            if (value.Entry?.HasProperties == true)
            {
                IsTextView = false;
                // TODO load property
            }
        }
        else if (value.Entry is not null)
        {
            // var path = Url + "/" + value.Path;
            Dispatcher.UIThread.InvokeAsync(async () => { await TryCatFile(value.Entry); });
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
        Manager.Default.Send(new OnNotification(new Notification
        {
            Title = "Error",
            Content = $"Failed to query: {e.HumanReadableMessage}",
            Type = NotificationType.Error,
        }), Manager.MainWindowToken);
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
    
    protected override async Task OnLoaded()
    {
        await base.OnLoaded();
        
        var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;
        
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
                DirentHasProperties: true, 
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
                RetryCommand = LoadedCommand,
                RetryCommandParameter = null
            };
        }
        finally 
        {
            TreeViewState = LoadingOrErrorState.MakeNone();
        }

    }
}