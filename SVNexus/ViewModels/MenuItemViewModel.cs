using System.Collections.ObjectModel;
using System.Windows.Input;

namespace SVNexus.ViewModels;

public class MenuItemViewModel
{
    public object? Icon { get; set; }
    
    public object? Header { get; set; }
    public ICommand? Command { get; set; }
    public object? CommandParameter { get; set; }

    // 子菜单
    public ObservableCollection<MenuItemViewModel> Children { get; set; } = [];

    // 可选：是否可见、是否启用、是否勾选
    public bool IsEnabled { get; set; } = true;
}
