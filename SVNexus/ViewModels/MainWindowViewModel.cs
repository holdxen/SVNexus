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
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;

namespace SVNexus.ViewModels;

public partial class MainWindowViewModel : ViewModelBase, IDisposable, IRecipient<OnRemoveTab>, IRecipient<OnOpenRepository>, IRecipient<OnAddTab>, IRecipient<OnRemoveTabByLocalViewModel>, IRecipient<OnRemoveTabByContent>
{
    
    public partial class TabItemViewViewModel: ViewModelLite
    {

        [ObservableProperty]
        private bool _closable = true;
    
        [ObservableProperty]
        private string _text = "";
    

        [RelayCommand]
        private async Task OnClose()
        {
        }
    
    
        [ObservableProperty]
        private object? _content;
        
        
        [ObservableProperty]
        public partial bool IsSelected { get; set; }
    
    
    }
    

    public MainWindowViewModel()
    {
        this.RegisterAllMessages(Manager.MainWindow);
        AddTab();
    }
    
    // private readonly IAppState _appState;


    // [ObservableProperty]
    // private Rect _tabRect;


    // partial void OnTabRectChanged(Rect value)
    // {
    // }

    // public int TabIndex { get => _appState.TabIndex; set => _appState.TabIndex = value; }
    
    
    // public ObservableCollection<TabItemModel> Tabs { get => _appState.Tabs; set => _appState.Tabs = value; }
    
    [ObservableProperty]
    private int _selectedIndex = -1;

    public ObservableCollection<TabItemViewViewModel> Tabs { get; set; } = [];

    // public MainWindowViewModel(IAppState appState)
    // {
    //     _appState = appState;
    //     for (var i = 0; i < 5; i++)
    //     {
    //         Tabs.Add(new TabItemModel()
    //         {
    //             Text = "Welcome to Avalonia!",
    //         });
    //     }
    // }
    //

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


    private TabItemViewViewModel NewTab()
    {
        return new TabItemViewViewModel
        {
            Text = "Welcome",
            Content = new WelcomeViewModel()
        };
    }

    private void AddTab(TabItemViewViewModel tab)
    {
        Tabs.Add(tab);
        SelectedIndex = Tabs.Count - 1;
    }

    // [RelayCommand]
    private void AddTab()
    {
        var tab = new TabItemViewViewModel
        {
            Text = "Welcome",
            Content = new WelcomeViewModel()
        };
        
        AddTab(tab);
       
    }

    public void Receive(OnRemoveTab message)
    {
        var index = Tabs.IndexOf(message.Value);
        if (index == -1 || SelectedIndex != index)
        {
            return;
        }

        if (index != 0)
        {
            index--;
        }
        Tabs.Remove(message.Value);
        SelectedIndex = index;
    }

    private void ReleaseUnmanagedResources()
    {
        Manager.MainWindow.UnregisterAll(this);
    }

    public void Dispose()
    {
        ReleaseUnmanagedResources();
        GC.SuppressFinalize(this);
    }

    public void Receive(OnOpenRepository message)
    {
        var workingCopyView = WorkingCopyViewModel.Create(message.Value);

        var tab = new TabItemViewViewModel()
        {
            Closable = true,
            Content = workingCopyView,
            Text = Path.GetFileName(message.Value.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar)),
        };
        
        AddTab(tab);
    }

    public void Receive(OnAddTab message)
    {
        Tabs.Add(message.Value);
        SelectedIndex = Tabs.Count - 1;
    }

    public void Receive(OnRemoveTabByLocalViewModel message)
    {
        foreach (var tab in Tabs)
        {
            if (tab.Content is not WorkingCopyViewModel workingCopyViewModel ||
                workingCopyViewModel.LocalViewModel != message.Value) continue;
            var index = Tabs.IndexOf(tab);
            if (index == SelectedIndex)
            {
                SelectedIndex = SelectedIndex == 1 ? 1 : SelectedIndex - 1;
            }
            Tabs.Remove(tab);
            break;
        }
    }

    public void Receive(OnRemoveTabByContent message)
    {
        foreach (var tab in Tabs)
        {
            if (Equals(tab.Content, message.Value))
            {
                Tabs.Remove(tab);
            }
            break;
        }
    }

    ~MainWindowViewModel()
    {
        ReleaseUnmanagedResources();
    }
}