using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using Ursa.Controls;

namespace SVNexus.ViewModels.WorkingCopy.Changes;

public partial class ChangesListViewModel : ViewModelBase
{
    /// <inheritdoc/>
    public ChangesListViewModel(ViewModelBase parent) : base(parent)
    {
        SelectedItems?.CollectionChanged += (sender, args) =>
        {
            NotifySelectedItemsChanged();
        };
    }

    public partial class MenuIconViewModel: ViewModelBase
    {
        [ObservableProperty]
        public partial string? Source { get; set; }

        [ObservableProperty] public partial bool Themable { get; set; } = true;
    }

    public partial class ListItemViewModel : StatusEntryItemViewModel
    {
        public List<MenuItemViewModel>? MenuItems
        {
            get
            {
                List<MenuItemViewModel> menuItems = [];
                switch (Entry.NodeStatus)
                {
                    case WorkingCopyStatus.None:
                        break;
                    case WorkingCopyStatus.Unversioned:
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
                    case WorkingCopyStatus.Normal:
                        break;
                    case WorkingCopyStatus.Added:
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
                    case WorkingCopyStatus.Missing:
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
                    case WorkingCopyStatus.Deleted:
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
                    case WorkingCopyStatus.Replaced:
                        break;
                    case WorkingCopyStatus.Modified:
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
                    case WorkingCopyStatus.Merged:
                        menuItems.Add(new MenuItemViewModel()
                        {
                            Header = "Merge",
                        });
                        break;
                    case WorkingCopyStatus.Conflicted:
                        break;
                    case WorkingCopyStatus.Ignored:
                        break;
                    case WorkingCopyStatus.Obstructed:
                        break;
                    case WorkingCopyStatus.External:
                        break;
                    case WorkingCopyStatus.Incomplete:
                        break;
                    default:
                        throw new ArgumentOutOfRangeException();
                }
                
                return menuItems.Count == 0 ? null : menuItems;
            }
        }

        // partial void OnIsCheckedChanged(bool value)
        // {
        //     SendMessage(new OnItemCheckStateChanged()
        //     {
        //         Item = this,
        //         Checked = value
        //     });
        // }


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
                using var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

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
            
            if (Entry.NodeStatus == WorkingCopyStatus.Unversioned)
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
                using var context = Engine.EngineBackend.Instance.SimpleContext(hostId);
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


    [ObservableProperty] 
    public partial ObservableCollection<ListItemViewModel> SelectedItems { get; set; } = [];
    
    public ObservableCollection<ListItemViewModel> Items { get; } = [];
    
    
    [ObservableProperty]
    public partial bool SearchMode { get; set; } 
    
    [ObservableProperty]
    public partial string SearchText { get; set; } = string.Empty;

    partial void OnSelectedItemChanged(ListItemViewModel? value)
    {
        SendMessage(new Messages.OnSelectedItemChanged(value?.Entry));
    }

    partial void OnSelectedItemsChanged(ObservableCollection<ListItemViewModel> value)
    {
        NotifySelectedItemsChanged();
    }

    public void Update(StatusEntry[] entries)
    {
        Items.Clear();

        ListItemViewModel? selectedItem = null;
        foreach (var entry in entries)
        {
            var item = new ListItemViewModel
            {
                Entry = entry,
                RelateTo = SendMessage(new OnGetWorkingCopyPath())
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
    
    }

    public void NotifySelectedItemsChanged()
    {
        SendMessage(new Messages.OnSelectedItemsChanged(SelectedItems.Select(i => i.Entry).ToList()));
    }
    
    [RelayCommand]
    private void Show()
    {
        NotifySelectedItemsChanged();
    }
}