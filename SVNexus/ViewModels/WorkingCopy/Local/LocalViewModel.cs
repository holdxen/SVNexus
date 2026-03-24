using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using Ursa.Controls;
using Exception = SVNexus.Generated.Exception;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalViewModel: ViewModelBase
{
    public partial class TreeItemViewModel: ViewModelBase, IRecipient<OnSetChecked>
    {
        [ObservableProperty]
        public partial string WorkingCopyPath { get; set; } = string.Empty;
        
        [ObservableProperty]
        public partial bool HasLoaded { get; set; }
        
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }

        public ObservableCollection<TreeItemViewModel> Children { get; set; } = [];
        
        public ObservableCollection<MenuItemViewModel> MenuItems { get; set; } = [];

        public required StatusEntry StatusEntry { get; set; }
        
        public bool HasChild => StatusEntry.NodeKind == NodeKind.Directory;
        
        public bool IsReal => StatusEntry.NodeStatus is not (NodeStatus.None or NodeStatus.Normal);
        
        public bool IsDelete => StatusEntry.NodeStatus == NodeStatus.Deleted;

        [ObservableProperty] public partial bool IsLoading { get; set; }
        
        [ObservableProperty]
        public partial bool IsChecked { get; set; }

        public string NodeKindIcon => StatusEntry.NodeKind.NodeKindIcon();


        public string Name => StatusEntry.Path.GetFileName();


        public string StatusToolTip => StatusEntry.NodeStatus.ToString();


        public string StatusIcon => StatusEntry.NodeStatus.NodeStatusIcon();
        

        partial void OnIsExpandedChanged(bool value)
        {
            if (value && !HasLoaded)
            {
                _ = LoadChildren();
            }
        }

        public async Task RefreshChildren()
        {
            if (IsLoading || !HasLoaded || !HasChild)
            {
                return;
            }
            IsLoading = true;
            var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;
            
            var context = Engine.Engine.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(StatusEntry.Path, new Revision.Working(), Depth.Immediates, true, true, true, false, false, false, null);

            var children = new List<TreeItemViewModel>();
            var receiver = new StatusReceiverDelegate()
            {
                OnStatusEntryAction = entry =>
                {
                    try
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            if (entry.Path == StatusEntry.Path)
                            {
                                return;
                            }

                            var index = Children.FindIndex(i => i.StatusEntry.Path == entry.Path);
                            if (index >= 0)
                            {
                                Children[index].StatusEntry = entry;
                                children.Add(Children[index]);
                            }
                            else
                            {
                                children.Add(new TreeItemViewModel()
                                {
                                   StatusEntry = entry,
                                   Token = Token
                                });
                            }
                            
                            Children.Clear();
                            Children.AddRange(children);
                        });
                    }
                    catch (System.Exception e)
                    {
                        Console.WriteLine(e);
                    }
                }
            };
            
            await context.StatusNext(statusOptions, receiver);
            
            IsLoading = true;
            HasLoaded = true;
        }


        public async Task LoadChildren()
        {
            if (HasLoaded || IsLoading || !HasChild)
            {
                return;
            }
            IsLoading = true;
            var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;
            
            var context = Engine.Engine.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(StatusEntry.Path, new Revision.Working(), Depth.Immediates, true, true, true, false, false, false, null);

            var receiver = new StatusReceiverDelegate()
            {
                OnStatusEntryAction = entry =>
                {
                    try
                    {
                        Dispatcher.UIThread.Invoke(() =>
                        {
                            if (entry.Path == StatusEntry.Path)
                            {
                                return;
                            }
                
                
                            Children.Add(new TreeItemViewModel()
                            {
                                StatusEntry = entry,
                                Token = Token
                            });
                        });
                    }
                    catch (System.Exception e)
                    {
                        Console.WriteLine(e);
                    }
                }
            };
            
            await context.StatusNext(statusOptions, receiver);

            HasLoaded = true;
            IsLoading = false;
        }

        public void Receive(OnSetChecked message)
        {
            IsChecked = message.Value;
        }
    }

    [ObservableProperty]
    public partial string WorkingCopyPath { get; set; } = string.Empty;
    
    public ObservableCollection<TreeItemViewModel> TreeItems { get; set; } = [];
    
    [ObservableProperty]
    public partial bool ShowRoot { get; set; }
    
    [ObservableProperty]
    public partial bool IsLoading { get; set; }
    
    private Dictionary<string, StatusEntry> _checkedItems = [];
    private HashSet<string> _expandedItems = [];

    private TreeItemViewModel? _root;
    
    private readonly SingleTaskQueue _singleTaskQueue = new();


    [RelayCommand]
    private void CheckAll()
    {
        Manager.Default.Send(new OnSetChecked(true), Token);
    }
    
    [RelayCommand]
    private void ClearAll()
    {
        Manager.Default.Send(new OnSetChecked(false), Token);
    }

    partial void OnShowRootChanged(bool value)
    {
        if (_root is null)
        {
            return;
        }
        
        TreeItems.Clear();
        if (value)
        {
            TreeItems.Add(_root);
        }
        else
        {
            TreeItems.AddRange(_root.Children);
        }
    }

    private async Task RefreshItem(TreeItemViewModel item)
    {
        await item.RefreshChildren();
        foreach (var child in item.Children)
        {
            _ = Dispatcher.UIThread.InvokeAsync(async () =>
            {
                await RefreshItem(child);
            });
        }
    }

    [RelayCommand]
    private async Task Refresh()
    {
        await _singleTaskQueue.Run(async token =>
        {
            if (_root is null)
            {
                if (!IsLoading)
                {
                    token.ThrowIfCancellationRequested();
                    await LoadRoot();
                }
            }
            token.ThrowIfCancellationRequested();

            if (_root is not null)
            {
                await RefreshItem(_root);
            }
        });
    }

    private async Task LoadAllChildren(TreeItemViewModel item)
    {
        if (item is { HasLoaded: false, HasChild: true })
        {
            await item.LoadChildren();
        }

        foreach (var child in item.Children)
        {
            child.Token = Token;
            _ = Dispatcher.UIThread.InvokeAsync(async () =>
            {
                await LoadAllChildren(child);
            });
        }
    }

    [RelayCommand]
    private async Task LoadAll()
    {
        await _singleTaskQueue.Run(async token =>
        {
            if (_root is null && !IsLoading)
            { 
                token.ThrowIfCancellationRequested();
                await LoadRoot();
            }

            if (_root is null)
            {
                return;
            }
            token.ThrowIfCancellationRequested();
            await LoadAllChildren(_root);
        }, false);
    }


    private async Task LoadRoot()
    {
        IsLoading = true;
        try
        {
            var hostId = Manager.Default.Send(new OnGetDialogHostId(), Token).Response;

            var context = Engine.Engine.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(
                WorkingCopyPath,
                new Revision.Working(),
                Depth.Immediates,
                true,
                true,
                true,
                false,
                true,
                false,
                null);

            var result = await context.Status(statusOptions);

            var children = new List<TreeItemViewModel>();

            foreach (var entry in result.Entries)
            {
                if (entry.Path == WorkingCopyPath)
                {
                    _root = new TreeItemViewModel()
                    {
                        StatusEntry = entry,
                        Token = Token
                    };
                }
                else
                {
                    children.Add(new TreeItemViewModel()
                    {
                        StatusEntry = entry,
                        Token = Token
                    });
                }
            }

            _root?.Children.AddRange(children);
            _root?.HasLoaded = true;
            _root?.IsExpanded = true;
            OnShowRootChanged(ShowRoot);
        }
        catch (System.Exception e)
        {
            Manager.Default.Send(new OnShowToast()
            {
                Content = $"Failed to load working copy: {e.HumanReadableMessage}",
                Type = NotificationType.Error
            }, Manager.MainWindowToken);
        }
        finally
        {
            IsLoading = false;
        }
    }

    protected override async Task OnLoaded()
    {
        await base.OnLoaded();
        await _singleTaskQueue.Run(async token => await LoadRoot());
    }
}