using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using Avalonia.Threading;
using AvaloniaEdit.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using CommunityToolkit.Mvvm.Messaging.Messages;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalViewModel(ViewModelBase parent): ViewModelBase(parent), IRecipient<LocalViewModel.OnCreatedItem>
{


    public class OnCreatedItem(TreeItemViewModel value) : ValueChangedMessage<TreeItemViewModel>(value);
    
    
    public partial class TreeItemViewModel(ViewModelBase? parent): ViewModelBase(parent)//, IRecipient<OnSetChecked>, IRecipient<OnSetExpanded>
    {

        // private readonly IServiceProvider _serviceProvider;
        //
        // private readonly Services.ITabService _tabService;
        

        // private readonly Services.TypeService _typeService;
        //
        // public TreeItemViewModel(IServiceProvider serviceProvider)
        // {
        //     _serviceProvider = serviceProvider;
        //     _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
        //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
        //     
        //     Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
        // }
        
        [ObservableProperty]
        public partial bool HasLoaded { get; set; }
        
        [ObservableProperty]
        public partial bool IsExpanded { get; set; }

        [ObservableProperty]
        public partial ObservableCollection<TreeItemViewModel> Children { get; set; } = [];
        
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
            try
            {

                // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
                var hostId = SendMessage(new OnGetDialogHostId());

                var context = Engine.Engine.Instance.SimpleContext(hostId);

                var statusOptions = new StatusOptions(StatusEntry.Path, new Revision.Working(), Depth.Immediates, true,
                    true, true, false, false, false, null);

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
                                    var item = new TreeItemViewModel(parent)
                                    {
                                        StatusEntry = entry
                                    };
                                    children.Add(item);
                                    
                                    SendMessage(new OnCreatedItem(item));
                                }

                                Children = new ObservableCollection<TreeItemViewModel>(children);
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
            }
            catch (System.Exception e)
            {
                Console.WriteLine(e);
            }
            finally
            {
                IsLoading = false;
            }
            
        }


        public async Task LoadChildren()
        {
            if (HasLoaded || IsLoading || !HasChild)
            {
                return;
            }
            IsLoading = true;
            // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
            
            var hostId = SendMessage(new OnGetDialogHostId());
            
            var context = Engine.Engine.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(StatusEntry.Path, new Revision.Working(), Depth.Immediates, true, false, false, false, false, false, null);

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


                            var item = new TreeItemViewModel(Parent)
                            {
                                StatusEntry = entry,
                            };
                            Children.Add(item);
                            SendMessage(new OnCreatedItem(item));
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
    }

    // [ObservableProperty]
    // public partial string WorkingCopyPath { get; set; } = string.Empty;

    private readonly List<WeakReference<TreeItemViewModel>> _weakTreeItems = [];
    
    public ObservableCollection<TreeItemViewModel> TreeItems { get; set; } = [];
    
    [ObservableProperty]
    public partial bool ShowRoot { get; set; }
    
    [ObservableProperty]
    public partial bool IsLoading { get; set; }
    
    private Dictionary<string, StatusEntry> _checkedItems = [];
    private HashSet<string> _expandedItems = [];

    private TreeItemViewModel? _root;
    
    private readonly SingleTaskQueue _singleTaskQueue = new();

    // private readonly Services.ITabService _tabService;
    //
    // private readonly Services.WorkingCopyViewService _workingCopyViewService;
    //
    // private readonly IServiceProvider _serviceProvider;
    //
    // private readonly Services.TypeService _typeService;

    // public LocalViewModel(IServiceProvider serviceProvider)
    // {
    //     _serviceProvider = serviceProvider;
    //     _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
    //     _workingCopyViewService = serviceProvider.GetRequiredService<Services.WorkingCopyViewService>();
    //     
    //     
    //
    //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
    //     
    //     Manager.Default.RegisterAllMessages(this, this.GetToken(_typeService));
    // }

    private void CleanWeakTreeItems()
    {
        ForEachWeakTreeItem(item => {});
    }

    private void ForEachWeakTreeItem(Action<TreeItemViewModel> action)
    {
        for (var i = _weakTreeItems.Count - 1; i >= 0; i--)
        {
            if (_weakTreeItems[i].TryGetTarget(out var item))
            {
                action(item);
            }
            else
            {
                _weakTreeItems.RemoveAt(i);
            }
        }
    }

    [RelayCommand]
    private void CollapseAll()
    {
        // Manager.Default.Send(new OnSetExpanded(false), _typeService.Get<TreeItemViewModel>());
        ForEachWeakTreeItem(item => item.IsExpanded = false);
    }

    [RelayCommand]
    private void ExpandAll()
    {
        // Manager.Default.Send(new OnSetExpanded(true), _typeService.Get<TreeItemViewModel>());
        ForEachWeakTreeItem(item => item.IsExpanded = true);
    }

    [RelayCommand]
    private void CheckAll()
    {
        // Manager.Default.Send(new OnSetChecked(true), _typeService.Get<TreeItemViewModel>());
        ForEachWeakTreeItem(item => item.IsChecked = true);
    }
    
    [RelayCommand]
    private void ClearAll()
    {
        // Manager.Default.Send(new OnSetChecked(false), _typeService.Get<TreeItemViewModel>());
        ForEachWeakTreeItem(item => item.IsChecked = false);
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
            CleanWeakTreeItems();
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
            CleanWeakTreeItems();
        }, false);
    }


    private async Task LoadRoot()
    {
        IsLoading = true;
        try
        {
            // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
            var hostId = SendMessage(new OnGetDialogHostId());
            var path = SendMessage(new OnGetWorkingCopyPath());

            var context = Engine.Engine.Instance.SimpleContext(hostId);

            var statusOptions = new StatusOptions(
                path,
                new Revision.Working(),
                Depth.Immediates,
                true,
                false,
                false,
                false,
                true,
                false,
                null);

            var result = await context.Status(statusOptions);

            var children = new List<TreeItemViewModel>();

            foreach (var entry in result.Entries)
            {
                if (entry.Path == path)
                {
                    _root = new TreeItemViewModel(this)
                    {
                        StatusEntry = entry,
                    };
                    
                    Receive(new OnCreatedItem(_root));
                }
                else
                {
                    children.Add(new TreeItemViewModel(this)
                    {
                        StatusEntry = entry,
                    });
                    
                    Receive(new OnCreatedItem(children.Last()));
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

    [RelayCommand]
    private async Task OnLoaded()
    {
        await _singleTaskQueue.Run(async token => await LoadRoot());
    }

    public void Receive(OnCreatedItem message)
    {
        _weakTreeItems.Add(new WeakReference<TreeItemViewModel>(message.Value));
    }
}