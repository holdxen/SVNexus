using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Notifications;
using Avalonia.Controls.Shapes;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Inject;
using SVNexus.Messages;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesListViewModel(ViewModelBase? parent = null): ViewModelBase(parent), IRecipient<ChangesListViewModel.OnItemCheckStateChanged>
{
    // private readonly IServiceProvider _serviceProvider;
    public partial class MenuIconViewModel: ViewModelBase
    {
        [ObservableProperty]
        public partial string? Source { get; set; }

        [ObservableProperty] public partial bool Themable { get; set; } = true;
    }

    public class OnItemCheckStateChanged
    {
        public required ListItemViewModel Item { get; set; }
        public required bool Checked { get; set; }
    }

    public partial class ListItemViewModel : ViewModelBase
    {
        [ObservableProperty]
        public partial bool IsChecked { get; set; }


        public string? ContainsDirectory
        {
            get
            {
                var path = SendMessage(new OnGetWorkingCopyPath());
                if (path == StatusEntry.Path)
                {
                    return null;
                }
                var contain = StatusEntry.Path.TrimStartString(path)
                    .TrimStartPathSeparatorChar().GetDirectoryName();
                return string.IsNullOrEmpty(contain) ? "/" : contain;
            }
        }

        public string Name => StatusEntry.Path.GetFileName();
    

        public string AbsolutePath => StatusEntry.Path;



        public string PathSvgIcon => StatusEntry.NodeKind.NodeKindIcon();
    
        public string StatusSvgIcon => StatusEntry.NodeStatus.NodeStatusIcon();
    
    
        public string StatusToolTip => StatusEntry.NodeStatus.ToString();

    

        [ObservableProperty] 
        [NotifyPropertyChangedFor(nameof(Name))]
        [NotifyPropertyChangedFor(nameof(ContainsDirectory))]
        [NotifyPropertyChangedFor(nameof(AbsolutePath))]
        public required partial StatusEntry StatusEntry { get; set; }


        // [ObservableProperty]
        // [NotifyPropertyChangedFor(nameof(Name))]
        // [NotifyPropertyChangedFor(nameof(ContainsDirectory))]
        // public partial string WorkingCopyPath { get; set; } = string.Empty;


        public List<MenuItemViewModel>? MenuItems
        {
            get
            {
                List<MenuItemViewModel> menuItems = [];
                switch (StatusEntry.NodeStatus)
                {
                    case NodeStatus.None:
                        break;
                    case NodeStatus.Unversioned:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Add",
                            Command = AddCommand,
                            Icon = new MenuIconViewModel()
                            {
                                Source = "Icons.OperationAdd"
                            }
                        });
                        break;
                    case NodeStatus.Normal:
                        break;
                    case NodeStatus.Added:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Revert",
                            Command = RevertCommand,
                            Icon = new MenuIconViewModel()
                            {
                                Source = "Icons.OperationRevert"
                            }
                        });
                        break;
                    case NodeStatus.Missing:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Revert",
                            Command = RevertCommand,
                            Icon = new MenuIconViewModel()
                            {
                                Source =  "Icons.OperationRevert"
                            }
                        });
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Delete",
                            Command = DeleteCommand,
                            Icon = new MenuIconViewModel()
                            {
                                Source =   "Icons.OperationDelete"
                            }
                        });
                        break;
                    case NodeStatus.Deleted:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Revert",
                            Command = RevertCommand,
                            Icon = new MenuIconViewModel()
                            {
                                Source = "Icons.OperationRevert"
                            }
                        });
                        break;
                    case NodeStatus.Replaced:
                        break;
                    case NodeStatus.Modified:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Revert",
                            Command = RevertCommand,
                            Icon = new MenuIconViewModel()
                            {
                                Source = "Icons.OperationRevert"
                            }
                        });
                        break;
                    case NodeStatus.Merged:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Merge",
                        });
                        break;
                    case NodeStatus.Conflicted:
                        break;
                    case NodeStatus.Ignored:
                        break;
                    case NodeStatus.Obstructed:
                        break;
                    case NodeStatus.External:
                        break;
                    case NodeStatus.Incomplete:
                        break;
                    default:
                        throw new ArgumentOutOfRangeException();
                }
                
                return menuItems.Count == 0 ? null : menuItems;
            }
        }

        // private readonly Services.ITabService _tabService;
        // private readonly Services.IWorkingCopyViewService _workingCopyViewService;
        // private readonly Services.TypeService _typeService;
        // public ListItemViewModel(IServiceProvider serviceProvider)
        // {
        //     _tabService = serviceProvider.GetRequiredService<Services.ITabService>();
        //     _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
        //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
        //     
        //     Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
        // }

        partial void OnIsCheckedChanged(bool value)
        {
            SendMessage(new OnItemCheckStateChanged()
            {
                Item = this,
                Checked = value
            });
        }


        [RelayCommand]
        private async Task Add()
        {
            if (Parent is null)
            {
                return;
            }
            
            var hostId = SendMessage(new OnGetDialogHostId()).Response;

            try
            {
                using var context = Engine.Engine.Instance.SimpleContext(hostId);

                var addOptions = new AddOptions(AbsolutePath, Depth.Empty, false, false, false, false);
                
                await context.Add(addOptions);
                SendMessage(new OnRefreshWorkingCopy());
            }
            catch (System.Exception e)
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = $"Failed to add: {e.HumanReadableMessage}",
                    Type = NotificationType.Error
                }, Manager.MainWindowToken);
            }
            
        }

        [RelayCommand]
        private async Task Delete()
        {
            
        }
        
        [RelayCommand]
        private async Task Revert()
        {

            if (Parent is null)
            {
                return;
            }

            var hostId = SendMessage(new OnGetDialogHostId()).Response;
            var path = SendMessage(new OnGetWorkingCopyPath()).Response;
            
            if (StatusEntry.NodeStatus == NodeStatus.Unversioned)
            {
                // var result = await MessageBox.ShowOverlayAsync(message: $"\n{AbsolutePath.TrimStartString(WorkingCopyPath).TrimStartPathSeparatorChar()} will be permanently deleted", 
                //     title: "Warning", hostId: hostId, MessageBoxIcon.Warning, MessageBoxButton.YesNo);
                // if (result != MessageBoxResult.Yes)
                // {
                //     return;
                // }

                return;
            }

            var result = await MessageBox.ShowOverlayAsync(message: $"Whether to revert:\n{AbsolutePath.TrimStartString(path).TrimStartPathSeparatorChar()}", 
                title: "Question", hostId: hostId, MessageBoxIcon.Question, MessageBoxButton.YesNo);
            if (result != MessageBoxResult.Yes)
            {
                return;
            }
            try
            {
                using var context = Engine.Engine.Instance.SimpleContext(hostId);
                var revertOptions = new RevertOptions(
                    Paths: [AbsolutePath], 
                    Depth: Depth.Empty, 
                    Changelists: [], 
                    ClearChangelists: false, 
                    MetadataOnly: false, 
                    AddedKeepLocal: true);
            
                await context.Revert(revertOptions);
                // Manager.Default.Send(new OnRefreshWorkingCopy(), _typeService.Get<WorkingCopyViewModel>());
                SendMessage(new OnRefreshWorkingCopy());
            }
            catch (System.Exception e)
            {
                Manager.Default.Send(new OnShowToast()
                {
                    Content = $"Failed to revert {e.HumanReadableMessage}",
                    Type = NotificationType.Error
                }, Manager.MainWindowToken);
            }
        }


        // public required WeakReferenceMessenger Messenger { get; init; }



    }

    [ObservableProperty]
    public partial ListItemViewModel? SelectedItem { get; set; }
    
    
    public ObservableCollection<ListItemViewModel> Items { get; } = [];
    
    
    public Dictionary<string, StatusEntry> CheckedItems { get; set; } = [];
    
    [ObservableProperty]
    public partial bool SearchMode { get; set; } 
    
    [ObservableProperty]
    public partial string SearchText { get; set; } = string.Empty;

    [ObservableProperty]
    public partial bool? AllChecked { get; set; }

    private bool BlockSignal { get; set; }

    // private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    // private readonly Services.TypeService _typeService;
    // public ChangesListViewModel(IServiceProvider serviceProvider)
    // {
    //     _serviceProvider = serviceProvider;
    //     _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
    //     _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
    //     
    //     Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
    // }

    partial void OnAllCheckedChanged(bool? value)
    {
        if (BlockSignal)
        {
            return;
        }

        if (value is null)
        {
            return;
            
        }

        BlockSignal = true;

        foreach (var item in Items)
        {
            item.IsChecked = value.GetValueOrDefault();
        }

        BlockSignal = false;

    }

    

    
    
    

    partial void OnSelectedItemChanged(ListItemViewModel? value)
    {
        // Manager.Default.Send(new OnSelectedItemChanged(value?.StatusEntry), _typeService.Get<ChangesViewModel>());
        SendMessage(new OnSelectedItemChanged(value?.StatusEntry));
    }

    // public void Receive(OnLocalListItemSelected message)
    // {
    //     if (message.IsSelected)
    //     {
    //         SelectedItems.Add(message.ItemModel.StatusEntry.Path);
    //     }
    //     else
    //     {
    //         SelectedItems.RemoveAll(model => message.ItemModel.StatusEntry.Path == model);
    //     }
    // }

    public void Update(StatusEntry[] entries)
    {
        Items.Clear();

        ListItemViewModel? selectedItem = null;
        foreach (var entry in entries)
        {
            var item = new ListItemViewModel
            {
                StatusEntry = entry,
                IsChecked = CheckedItems.ContainsKey(entry.Path),
            };
            // item.ItemCheckStateChanged += OnItemCheckStateChanged;
            if (SelectedItem?.AbsolutePath == entry.Path)
            {
                selectedItem = item;
            }
        
            Items.Add(item);

            item.Parent = this;
        }

        SelectedItem = selectedItem;
    
        CheckedItems = Items.Where(item => item.IsChecked).ToDictionary(i => i.StatusEntry.Path, i => i.StatusEntry);
    
        UpdateAllChecked();
    }
    //
    // private void OnItemCheckStateChanged(ListItemViewModel itemModel, bool check)
    // {
    //     if (check)
    //     {
    //         CheckedItems.Add(itemModel.StatusEntry.Path, itemModel.StatusEntry);
    //     }
    //     else
    //     {
    //         CheckedItems.Remove(itemModel.StatusEntry.Path);
    //     }
    //     
    //     UpdateAllChecked();
    // }
    
    private void UpdateAllChecked()
    {
        if (BlockSignal)
        {
            return;
        }
        BlockSignal = true;
        if (Items.Count == 0)
        {
            AllChecked = false;
        }
        else if (CheckedItems.Count == Items.Count)
        {
            AllChecked = true;
        }
        else if (CheckedItems.Count == 0)
        {
            AllChecked = false;
        }
        else
        {
            AllChecked = null;
        }
        BlockSignal = false;
        Console.Write("Set All checked: {0}, Checked={1} Item={2}", AllChecked, CheckedItems.Count, Items.Count);
    }

    public void Receive(OnItemCheckStateChanged message)
    {
        if (message.Checked)
        {
            CheckedItems.Add(message.Item.StatusEntry.Path, message.Item.StatusEntry);
        }
        else
        {
            CheckedItems.Remove(message.Item.StatusEntry.Path);
        }
        
        UpdateAllChecked();
    }
}