using System.Collections.ObjectModel;
using System.Windows.Input;
using Avalonia.Controls;
using CommunityToolkit.Mvvm.ComponentModel;

namespace SVNexus.ViewModels;

public partial class MenuItemViewModel: ObservableObject
{
    [ObservableProperty]
    public partial object? Icon { get; set; }
    
    [ObservableProperty]
    public partial object? Header { get; set; }
    
    [ObservableProperty]
    public partial ICommand? Command { get; set; }
    
    [ObservableProperty]
    public partial object? CommandParameter { get; set; }

    // 子菜单
    [ObservableProperty]
    public partial ObservableCollection<MenuItemViewModel> Children { get; set; } = [];

    // 可选：是否可见、是否启用、是否勾选
    [ObservableProperty]
    public partial bool IsEnabled { get; set; } = true;
    
    [ObservableProperty]
    public partial bool IsChecked { get; set; }
    
    
    [ObservableProperty]
    public partial MenuItemToggleType  ToggleType { get; set; } = MenuItemToggleType.None;
    
}
