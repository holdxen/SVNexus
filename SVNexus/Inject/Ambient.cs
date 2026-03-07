using System;
using Avalonia;
using Avalonia.Data;

namespace SVNexus.Inject;

public sealed class Ambient : AvaloniaObject
{
    public static readonly AttachedProperty<Guid> GuidProperty =
        AvaloniaProperty.RegisterAttached<Ambient, AvaloniaObject, Guid>(
            "Guid",
            defaultValue: Guid.Empty,
            inherits: true,                 // ✅ 关键：向子级继承
            defaultBindingMode: BindingMode.OneWay);

    public static void SetGuid(AvaloniaObject element, Guid value) =>
        element.SetValue(GuidProperty, value);

    public static Guid GetGuid(AvaloniaObject element) =>
        element.GetValue(GuidProperty);
    
    
    public static readonly AttachedProperty<string> WorkingCopyPathProperty =
        AvaloniaProperty.RegisterAttached<Ambient, AvaloniaObject, string>(
            "WorkingCopyPath",
            defaultValue: string.Empty,
            inherits: true,                 // ✅ 关键：向子级继承
            defaultBindingMode: BindingMode.OneWay);

    public static void SetWorkingCopyPath(AvaloniaObject element, string value) =>
        element.SetValue(WorkingCopyPathProperty, value);

    public static string GetWorkingCopyPath(AvaloniaObject element) =>
        element.GetValue(WorkingCopyPathProperty);
    
    
    public static readonly AttachedProperty<string?> DialogHostIdProperty =
        AvaloniaProperty.RegisterAttached<Ambient, AvaloniaObject, string?>(
            "DialogHostId",
            defaultValue: null,
            inherits: true,                 // ✅ 关键：向子级继承
            defaultBindingMode: BindingMode.OneWay);

    public static void SetDialogHostId(AvaloniaObject element, string? value) =>
        element.SetValue(DialogHostIdProperty, value);

    public static string? GetDialogHostId(AvaloniaObject element) =>
        element.GetValue(DialogHostIdProperty);
}
