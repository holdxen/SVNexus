using Avalonia;
using Avalonia.Controls;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Utils;

namespace SVNexus.ViewModels;

public partial class SvgIconViewModel(ViewModelBase? parent = null): ViewModelBase(parent)
{
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Icon))]
    public partial string? IconKey { get; set; }
    
    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(Icon))]
    public partial string? IconSource { get; set; }

    [ObservableProperty] public partial bool Themable { get; set; } = true;

    [ObservableProperty] public partial double Size { get; set; } = 24;

    public string Icon
    {
        get
        {
            if (IconKey is not null)
            {
                object? resource = null;
                Application.Current?.TryFindResource(IconKey, out resource);
                
                if (resource is string icon) return icon;
                
                Logger.Error($"Failed to find resource: {IconKey}");
            }
            else if (IconSource is not null)
            {
                return IconSource;
            }

            return string.Empty;
        }
    }
    
}