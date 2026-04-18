using System;
using System.Collections.ObjectModel;
using System.IO;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.ViewModels.WorkingCopy;
using Tabalonia.Events;

namespace SVNexus.ViewModels;

public partial class MainWindowViewModel : ViewModelBase, 
    IDisposable, 
    IRecipient<OnRemoveTab>, 
    IRecipient<OnOpenRepository>, 
    IRecipient<OnAddTab>, 
    IRecipient<OnGetSingleTaskQueue>,
    IRecipient<OnGetTabByWorkspaceRoot>,
    IRecipient<OnSwitchTab>
{
    
    public partial class TabItemViewModel(ViewModelBase parent): ViewModelBase(parent), IRecipient<OnGetDialogHostId>, IRecipient<OnRemoveTabModel>
    {
        [ObservableProperty]
        public partial Guid Id { get; set; } = Guid.NewGuid();
        
        [ObservableProperty]
        public partial string? DialogHostId { get; set; }

        [ObservableProperty]
        public partial bool Closable { get; set; } = true;

        [ObservableProperty]
        public partial string Text { get; set; } = string.Empty;

        [ObservableProperty]
        public partial object? Content { get; set; }

        [ObservableProperty]
        public partial bool IsSelected { get; set; }


        public void Receive(OnGetDialogHostId message)
        {
            message.Reply(DialogHostId);
        }

        public void Receive(OnRemoveTabModel message)
        {
            if (Parent is MainWindowViewModel mainWindowViewModel)
            {
                mainWindowViewModel.RemoveTab(this);
            }
        }
    }

    public MainWindowViewModel()
    {
        AddTab();
        Manager.Default.RegisterAllMessages(this, Manager.MainWindowToken);
    }

    [ObservableProperty]
    public partial int SelectedIndex { get; set; } = -1;
    
    public ObservableCollection<TabItemViewModel> Tabs { get; set; } = [];

    private readonly SingleTaskQueue _singleTaskQueue = new()
    {
        Single = false
    };


    partial void OnSelectedIndexChanged(int value)
    {
        var index = 0;
        foreach (var tab in Tabs)
        {
            tab.IsSelected = index == value;
            index++;
        }
    }

    
    public Func<object> NewItemFactory => NewTab;


    private TabItemViewModel NewTab()
    {
        return new TabItemViewModel(this)
        {
            Text = "Welcome",
        }.Apply(item =>
        {
            item.Content = new WelcomeViewModel(item);
        });
    }

    private void AddTab(TabItemViewModel tab)
    {
        Tabs.Add(tab);
        SelectedIndex = Tabs.Count - 1;
    }

    [RelayCommand]
    private void AddTab()
    {
        var tab = new TabItemViewModel(this)
        {
            Text = "Welcome",
        }.Apply(item =>
        {
            item.Content = new WelcomeViewModel(item);
        });
        
        AddTab(tab);
       
    }

    private void RemoveIndexItem(int index)
    {
        if (index == -1)
        {
            return;
        }

        Tabs.RemoveAt(index);

        if (SelectedIndex != index) return;
        
        if (index != 0)
        {
            index--;
        }
        SelectedIndex = index;

    }

    public void Receive(OnRemoveTab message)
    {
        var index = Tabs.FindIndex(e => e.Id == message.Value);
        RemoveIndexItem(index);
    }

    public void RemoveTab(TabItemViewModel tab)
    {
        var index = Tabs.FindIndex(e => e == tab);
        RemoveIndexItem(index);
    }

    private void ReleaseUnmanagedResources()
    {
        Manager.Default.UnregisterAllMessages(this);
    }

    public void Dispose()
    {
        ReleaseUnmanagedResources();
        GC.SuppressFinalize(this);
    }

    public void Receive(OnOpenRepository message)
    {

        var tab = new TabItemViewModel(this)
        {
            Closable = true,
            Text = message.Value.GetFileName(),
        }.Apply(item =>
        {
            item.Content = new WorkspaceViewModel(message.Value, item);
        });
        
        
        AddTab(tab);
    }

    public void Receive(OnAddTab message)
    {
        // message.Value.Parent = this;
        // Tabs.Add(message.Value);
        var tab = new TabItemViewModel(this)
        {
            Text = message.Name,
            Closable = message.Closable,
        }.Apply(item =>
        {
            item.Content = message.Factory(item);
        });
        Tabs.Add(tab);
        SelectedIndex = Tabs.Count - 1;
    }

    // public void Receive(OnRemoveTabByLocalViewModel message)
    // {
    //     foreach (var tab in Tabs)
    //     {
    //         if (tab.Content is not WorkingCopyViewModel workingCopyViewModel ||
    //             workingCopyViewModel.LocalViewModel != message.Value) continue;
    //         var index = Tabs.IndexOf(tab);
    //         if (index == SelectedIndex)
    //         {
    //             SelectedIndex = SelectedIndex == 1 ? 1 : SelectedIndex - 1;
    //         }
    //         Tabs.Remove(tab);
    //         break;
    //     }
    // }
    //
    // public void Receive(OnRemoveTabByContent message)
    // {
    //     foreach (var tab in Tabs)
    //     {
    //         if (Equals(tab.Content, message.Value))
    //         {
    //             Tabs.Remove(tab);
    //         }
    //         break;
    //     }
    // }

    public void Receive(OnGetSingleTaskQueue message)
    {
        message.Reply(_singleTaskQueue);
    }

    public void Receive(OnGetTabByWorkspaceRoot message)
    {
        foreach (var tab in Tabs)
        {
            if (tab.Content is not WorkspaceViewModel workspaceViewModel) continue;
            if (workspaceViewModel.WorkspaceRoot != message.Root) continue;
            message.Reply(tab);
            return;
        }
        message.Reply(null);
    }

    public void Receive(OnSwitchTab message)
    {
        var index = Tabs.IndexOf(message.Tab);
        if (index < 0)
        {
            message.Reply(false);
        }
        else
        {
            SelectedIndex = index;
            message.Reply(true);
        }
    }

    ~MainWindowViewModel()
    {
        ReleaseUnmanagedResources();
    }
}