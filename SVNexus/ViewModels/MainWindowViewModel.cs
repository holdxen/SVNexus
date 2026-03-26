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
using Microsoft.Extensions.DependencyInjection;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.ViewModels.WorkingCopy;
using Tabalonia.Events;

namespace SVNexus.ViewModels;

public partial class MainWindowViewModel : ViewModelBase, IDisposable, IRecipient<OnRemoveTab>, IRecipient<OnOpenRepository>, IRecipient<OnAddTab>//, IRecipient<OnRemoveTabByLocalViewModel>, IRecipient<OnRemoveTabByContent>
{
    
    public partial class TabItemViewModel: ViewModelLite
    {
        [ObservableProperty]
        public partial Guid Id { get; set; } = Guid.NewGuid();

        [ObservableProperty]
        public partial bool Closable { get; set; } = true;

        [ObservableProperty]
        public partial string Text { get; set; } = "";

        [RelayCommand]
        private async Task OnClose()
        {
        }


        [ObservableProperty]
        public partial object? Content { get; set; }

        [ObservableProperty]
        public partial bool IsSelected { get; set; }
        
        public IServiceScope? Scope { get; set; }
    
    
    }

    public MainWindowViewModel()
    {
        AddTab();
        Manager.Default.RegisterAllMessages(this, Manager.MainWindowToken);
    }

    [ObservableProperty]
    public partial int SelectedIndex { get; set; } = -1;
    
    public ObservableCollection<TabItemViewModel> Tabs { get; set; } = [];


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
        var scope = InjectionProvider.Provider.CreateScope();
        var tabService = scope.ServiceProvider.GetRequiredService<Services.ITabService>();
        return new TabItemViewModel
        {
            Id =  tabService.Token,
            Text = "Welcome",
            Content = scope.ServiceProvider.GetRequiredService<WelcomeViewModel>(),
            Scope = scope,
        };
    }

    private void AddTab(TabItemViewModel tab)
    {
        Tabs.Add(tab);
        SelectedIndex = Tabs.Count - 1;
    }

    [RelayCommand]
    private void AddTab()
    {
        var scope = InjectionProvider.Provider.CreateScope();
        var tabService = scope.ServiceProvider.GetRequiredService<Services.ITabService>();
        var tab = new TabItemViewModel
        {
            Id =  tabService.Token,
            Text = "Welcome",
            Content = scope.ServiceProvider.GetRequiredService<WelcomeViewModel>(),
            Scope = scope,
        };
        
        AddTab(tab);
       
    }

    private void RemoveIndexItem(int index)
    {
        if (index == -1 || SelectedIndex != index)
        {
            return;
        }

        var tab = Tabs[index];
        Tabs.RemoveAt(index);
        
        if (index != 0)
        {
            index--;
        }
        SelectedIndex = index;
        tab.Scope?.Dispose();
    }

    [RelayCommand]
    private void RemovedItem(TabClosedEventArgs args)
    {
        if (args.Item is TabItemViewModel tab)
        {
            tab.Scope?.Dispose();
        }
    }

    public void Receive(OnRemoveTab message)
    {
        var index = Tabs.FindIndex(e => e.Id == message.Value);
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
        // var workingCopyView = new WorkingCopyViewModel
        // {
        //     WorkingCopyPath = message.Value,
        // };

        var scope = InjectionProvider.Provider.CreateScope();

        scope.ServiceProvider.GetRequiredService<Services.WorkingCopyViewService>().WorkingCopyPath = message.Value;
        
        var content = scope.ServiceProvider.GetRequiredService<WorkingCopyViewModel>();
        var tabService = scope.ServiceProvider.GetRequiredService<Services.TabService>();

        var tab = new TabItemViewModel()
        {
            Id = tabService.Token,
            Closable = true,
            Content = content,
            Text = message.Value.GetFileName()
        };
        
        AddTab(tab);
    }

    public void Receive(OnAddTab message)
    {
        Console.WriteLine("On AddTab");
        Tabs.Add(message.Value);
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

    ~MainWindowViewModel()
    {
        ReleaseUnmanagedResources();
    }
}