using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalListViewModel: ViewModelBase//, IRecipient<LocalListViewModel.OnLocalListItemSelected>
{
    // private WeakReferenceMessenger Messenger { get; } = new();

    public required string WorkingCopyPath { get; init; }


    public event Action<object?, StatusEntry?>? SelectedItemChanged;
    
    
    

    // public LocalListViewModel()
    // {
    //     Messenger.Register(this);
    // }
    //
    
    
    
    // public class OnLocalListItemSelected
    // {
    //     public required bool IsSelected { get; init; }
    //     public required ListItemViewModel ItemModel { get; init; }
    // }

    public partial class ListItemViewModel : ViewModelLite
    {
        
        // public event Action<ListItemViewModel, bool>? ItemCheckStateChanged;
        
        public required LocalListViewModel Parent { get; init; }
        
        [ObservableProperty]
        public partial bool IsChecked { get; set; }

        partial void OnIsCheckedChanged(bool value)
        {
            
            // Messenger.Send(new OnLocalListItemSelected
            // {
            //     IsSelected = value,
            //     ItemModel = this
            // });
            
            // ItemCheckStateChanged?.Invoke(this, value);
            Parent.OnItemCheckStateChanged(this, value);
        }

        public string Text => StatusEntry.Path.TrimStart(WorkingCopyPath.ToCharArray());
    

        public string AbsolutePath => StatusEntry.Path;



        public string PathSvgIcon => StatusEntry.NodeKind.NodeKindIcon();
    
        public string StatusSvgIcon => StatusEntry.NodeStatus.NodeStatusIcon();
    
    
        public string StatusToolTip => StatusEntry.NodeStatus.ToString();

    

        [ObservableProperty] 
        [NotifyPropertyChangedFor(nameof(Text))]
        [NotifyPropertyChangedFor(nameof(AbsolutePath))]
        public required partial StatusEntry StatusEntry { get; set; }
    
    
        public required string WorkingCopyPath { get; init; }
    
    
        // public required WeakReferenceMessenger Messenger { get; init; }



    }

    public override bool KeepAlive { get; set; } = true;

    [ObservableProperty]
    public partial ListItemViewModel? SelectedItem { get; set; }
    
    
    public ObservableCollection<ListItemViewModel> Items { get; } = [];
    
    
    public List<string> SelectedItems { get; set; } = [];


    partial void OnSelectedItemChanged(ListItemViewModel? value)
    {
        SelectedItemChanged?.Invoke(this, value?.StatusEntry);
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
                WorkingCopyPath = WorkingCopyPath,
                // Messenger =  Messenger,
                StatusEntry = entry,
                IsChecked = SelectedItems.Contains(entry.Path),
                Parent = this
            };
            // item.ItemCheckStateChanged += OnItemCheckStateChanged;
            if (SelectedItem?.WorkingCopyPath == entry.Path)
            {
                selectedItem = item;
            }
            
            Items.Add(item);
        }

        SelectedItem = selectedItem;
        
        Console.WriteLine("Items: {0}", Items.Count);
        SelectedItems = Items.Where(item => item.IsChecked).Select(item => item.WorkingCopyPath).ToList();
    }

    private void OnItemCheckStateChanged(ListItemViewModel itemModel, bool check)
    {
        if (check)
        {
            SelectedItems.Add(itemModel.StatusEntry.Path);
        }
        else
        {
            SelectedItems.RemoveAll(model => itemModel.StatusEntry.Path == model);
        }    
    }
}