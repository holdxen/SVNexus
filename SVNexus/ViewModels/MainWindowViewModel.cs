using System;
using System.Collections.ObjectModel;
using System.Threading.Tasks;
using Avalonia;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;

namespace SVNexus.ViewModels;

public partial class MainWindowViewModel : ViewModelBase, IDisposable, IRecipient<OnRemoveTab>, IRecipient<OnOpenRepository>, IRecipient<OnAddTab>, IRecipient<OnRemoveTabByLocalViewModel>
{
    
    public partial class TabItemViewViewModel: ViewModelLite
    {

        public string TabId { get; set; } = string.Empty;
        
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
    
    
    }
    

    public MainWindowViewModel()
    {
        WeakReferenceMessenger.Default.Register<OnRemoveTab>(this);
        WeakReferenceMessenger.Default.Register<OnOpenRepository>(this);
        WeakReferenceMessenger.Default.Register<OnAddTab>(this);
        WeakReferenceMessenger.Default.Register<OnRemoveTabByLocalViewModel>(this);
        Tabs.Add(NewTab());
    }
    
    // private readonly IAppState _appState;


    [ObservableProperty]
    private Rect _tabRect;


    partial void OnTabRectChanged(Rect value)
    {
    }

    // public int TabIndex { get => _appState.TabIndex; set => _appState.TabIndex = value; }
    
    
    // public ObservableCollection<TabItemModel> Tabs { get => _appState.Tabs; set => _appState.Tabs = value; }
    
    [ObservableProperty]
    private int _selectedIndex;

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
    
    
    // #e5f3ff

    double CalculateTabWidth()
    {
        return (TabRect.Width - 100) / Tabs.Count;
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

    [RelayCommand]
    private void AddTab()
    {
        Console.WriteLine("Add tab");
        var tab = new TabItemViewViewModel
        {
            Text = "Welcome",
            Content = new WelcomeViewModel()
        };
        
        
        Tabs.Add(tab);
        
        var w = CalculateTabWidth();
        // var maxWidth = 235;
        // Console.WriteLine($"set width to w ${w}");
        //
        // Dispatcher.UIThread.Post(() =>
        //     {
        //         tab.Width = Math.Min(w, maxWidth);
        //     }
        // );
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
        WeakReferenceMessenger.Default.UnregisterAll(this);
    }

    public void Dispose()
    {
        ReleaseUnmanagedResources();
        GC.SuppressFinalize(this);
    }

    public void Receive(OnOpenRepository message)
    {
        
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

    ~MainWindowViewModel()
    {
        ReleaseUnmanagedResources();
    }
}