using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using System.Threading.Tasks;
using Avalonia.Controls.Notifications;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Engine;
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

    // public partial class MenuIconViewModel: ViewModelBase
    // {
    //     [ObservableProperty]
    //     public partial string? Source { get; set; }
    //
    //     [ObservableProperty] public partial bool Themable { get; set; } = true;
    // }

    public partial class ListItemViewModel : TargetItemViewModel
    {
        public override Type? LocateType { get; } = typeof(TargetItemViewModel);

        public new static ListItemViewModel From(StatusEntry statusEntry, bool absolute = false, string? relateTo = null)
        {
            // return FromFactory<ListItemViewModel>(statusEntry, absolute, relateTo).Apply(
            //     e => e.Entry = statusEntry
            // );
            
            return new ListItemViewModel().Apply(e => e.Initialize(statusEntry, absolute, relateTo));
        }

        public StatusEntry Entry { get; set; } = null!;

        public override void Initialize(StatusEntry statusEntry, bool absolute, string? relateTo)
        {
            base.Initialize(statusEntry, absolute, relateTo);
            Entry = statusEntry;
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
                using var context = Engine.EngineBackend.Instance.SimpleContext(hostId);

                var addOptions = new AddOptions(Path, Depth.Empty, false, false, false, false);
                
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

            var result = await OverlayMessageBox.ShowAsync(message: $"Whether to revert:\n{Path.TrimStartString(path).TrimStartPathSeparatorChar()}", 
                title: "Question", hostId: hostId, MessageBoxIcon.Question, MessageBoxButton.YesNo);
            if (result != MessageBoxResult.Yes)
            {
                return;
            }
            try
            {
                using var context = Engine.EngineBackend.Instance.SimpleContext(hostId);
                var revertOptions = new RevertOptions(
                    Paths: [Path], 
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
    [NotifyPropertyChangedFor(nameof(MenuItems))]
    public partial ObservableCollection<ListItemViewModel> SelectedItems { get; set; } = [];

    public List<MenuItemViewModel> MenuItems
    {
        get
        {

            if (SelectedItems.Count == 0)
            {
                return [];
            }

            List<MenuItemViewModel> menuItems = [];

            if (SelectedItems.Count == 1)
            {
                menuItems.Add(new MenuItemViewModel()
                {
                    Header = "Ignore",
                    Children = [
                        new MenuItemViewModel()
                        {
                            Header = $"Ignore {SelectedItems.First().FileName}"
                        }
                    ]
                });
            }
            
            
            
            
            
            
            
            return menuItems;
        }
    }
    
    [ObservableProperty]
    public partial ObservableCollection<ListItemViewModel> Items { get; set; } = [];
    
    
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
        List<ListItemViewModel> items = [];
        var relateTo = SendMessage(new OnGetWorkingCopyPath());
        foreach (var entry in entries)
        {
            var index = Items.FindIndex(x => x.Entry.Path == entry.Path);
            if (index < 0)
            {
                items.Add(ListItemViewModel.From(entry, false, relateTo));
            }
            else
            {
                items.Add(Items[index].Apply(e =>
                {
                    e.Initialize(entry, false, relateTo);
                }));
            }
        }

        Items = new ObservableCollection<ListItemViewModel>(items);
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