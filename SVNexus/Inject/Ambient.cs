using Avalonia;
using Avalonia.Data;

namespace SVNexus.Inject;

public sealed class Ambient : AvaloniaObject
{
    public static readonly AttachedProperty<MyOptions?> ScopeProperty =
        AvaloniaProperty.RegisterAttached<Ambient, AvaloniaObject, MyOptions?>(
            "Scope",
            defaultValue: null,
            inherits: true,                 // ✅ 关键：向子级继承
            defaultBindingMode: BindingMode.OneWay);

    public static void SetScope(AvaloniaObject element, MyOptions? value) =>
        element.SetValue(ScopeProperty, value);

    public static MyOptions? GetScope(AvaloniaObject element) =>
        element.GetValue(ScopeProperty);
}

public sealed record MyOptions(string ApiBase, int TimeoutMs);

public interface IWorkingCopy
{
    public string WorkingCopyPath { get; }
    
    public string DialogHostId { get; }
}