using CommunityToolkit.Mvvm.ComponentModel;

namespace SVNexus.Models;

public partial class PropertyValue<T>: ObservableObject
{
    public PropertyValue(T value)
    {
        Value = value;
    }

    [ObservableProperty]
    public partial T Value { get; set; }
}