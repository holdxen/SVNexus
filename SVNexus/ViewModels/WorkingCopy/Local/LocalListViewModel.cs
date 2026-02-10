using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Linq;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Extension;
using SVNexus.Generated;

namespace SVNexus.ViewModels.WorkingCopy.Local;

public partial class LocalListViewModel: ViewModelBase, IRecipient<LocalListViewModel.OnLocalListItemSelected>
{
    private WeakReferenceMessenger Messenger { get; } = new();
    

    public required string WorkingCopyPath { get; set; } 

    public LocalListViewModel()
    {
        Messenger.Register(this);
    }
    
    
    
    
    public class OnLocalListItemSelected
    {
        public required bool IsSelected { get; init; }
        public required ListItemViewModel ItemView { get; init; }
    }

    public partial class ListItemViewModel : ViewModelLite
    {
        [ObservableProperty]
        public partial bool IsSelected { get; set; }

        partial void OnIsSelectedChanged(bool value)
        {
            Messenger.Send(new OnLocalListItemSelected
            {
                IsSelected = value,
                ItemView = this
            });
        }

        public string Text => StatusEntry.Path.TrimStart(WorkingCopyPath.ToCharArray());
    

        private string AbsolutePath => StatusEntry.Path;



        public string PathSvgIcon => StatusEntry.NodeKind.NodeKindIcon();
    
        public string StatusSvgIcon => StatusEntry.NodeStatus.NodeStatusIcon();
    
    
        public string StatusToolTip => StatusEntry.NodeStatus.ToString();

    

        [ObservableProperty] 
        [NotifyPropertyChangedFor(nameof(Text))]
        [NotifyPropertyChangedFor(nameof(AbsolutePath))]
        public required partial StatusEntry StatusEntry { get; set; }
    
    
        public required string WorkingCopyPath { get; init; }
    
    
        public required WeakReferenceMessenger Messenger { get; init; }



    }
    
    
    [ObservableProperty]
    private ListItemViewModel?  _selectedItem;

    public ObservableCollection<ListItemViewModel> Items { get; } = [];
    
    
    public List<string> SelectedItems { get; set; } = [];

    public void Receive(OnLocalListItemSelected message)
    {
        if (message.IsSelected)
        {
            SelectedItems.Add(message.ItemView.WorkingCopyPath);
        }
        else
        {
            SelectedItems.RemoveAll(model => Equals(message.ItemView.WorkingCopyPath, model));
        }
    }

    public void Update(StatusEntry[] entries)
    {
        Items.Clear();

        ListItemViewModel? selectedItem = null;
        foreach (var entry in entries)
        {
            var item = new ListItemViewModel
            {
                WorkingCopyPath = WorkingCopyPath,
                Messenger =  Messenger,
                StatusEntry = entry,
                IsSelected = SelectedItems.Contains(entry.Path)
            };
            if (SelectedItem?.WorkingCopyPath == entry.Path)
            {
                selectedItem = item;
            }
            
            Items.Add(item);
        }

        SelectedItem = selectedItem;
        
        Console.WriteLine("Items: {0}", Items.Count);
        SelectedItems = Items.Where(item => item.IsSelected).Select(item => item.WorkingCopyPath).ToList();
    }
}