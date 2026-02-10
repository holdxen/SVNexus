using System;
using System.Windows.Input;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;

namespace SVNexus.Components;

public partial class SvgButton : UserControl
{
    public SvgButton()
    {
        InitializeComponent();
    }
    
    
    public static readonly StyledProperty<string?> SourceProperty =
        AvaloniaProperty.Register<SvgButton, string?>(nameof(Source));

    public string? Source
    {
        get => GetValue(SourceProperty);
        set => SetValue(SourceProperty, value);
    }
    
    
        // 1) Command / CommandParameter 属性（可被样式/绑定）
    public static readonly StyledProperty<ICommand?> CommandProperty =
        AvaloniaProperty.Register<SvgButton, ICommand?>(nameof(Command));

    public static readonly StyledProperty<object?> CommandParameterProperty =
        AvaloniaProperty.Register<SvgButton, object?>(nameof(CommandParameter));

    public ICommand? Command
    {
        get => GetValue(CommandProperty);
        set => SetValue(CommandProperty, value);
    }

    public object? CommandParameter
    {
        get => GetValue(CommandParameterProperty);
        set => SetValue(CommandParameterProperty, value);
    }

    // 可选：暴露 Click 事件（有些人喜欢同时支持）
    public event EventHandler<RoutedEventArgs>? Click;

    private bool _pressed;

    static SvgButton()
    {
        // 可选：如果你想让它可聚焦、支持键盘触发（Space/Enter）
        FocusableProperty.OverrideDefaultValue<SvgButton>(true);
    }

    protected override void OnPointerPressed(PointerPressedEventArgs e)
    {
        base.OnPointerPressed(e);

        if (!e.GetCurrentPoint(this).Properties.IsLeftButtonPressed) return;
        _pressed = true;
        e.Pointer.Capture(this); // 捕获指针，行为更像 Button
        e.Handled = true;
    }

    protected override void OnPointerReleased(PointerReleasedEventArgs e)
    {
        base.OnPointerReleased(e);

        if (!_pressed) return;
        _pressed = false;

        // 释放捕获
        if (Equals(e.Pointer.Captured, this))
            e.Pointer.Capture(null);

        // 只有在控件区域内抬起才算“点击”
        var pt = e.GetPosition(this);
        var inside = pt.X >= 0 && pt.Y >= 0 && pt.X <= Bounds.Width && pt.Y <= Bounds.Height;

        if (inside)
            RaiseClick(e);

        e.Handled = true;
    }

    protected override void OnPointerCaptureLost(PointerCaptureLostEventArgs e)
    {
        base.OnPointerCaptureLost(e);
        _pressed = false;
    }

    protected override void OnKeyDown(KeyEventArgs e)
    {
        base.OnKeyDown(e);

        // 键盘触发：Space/Enter
        if (e.Key is not (Key.Space or Key.Enter)) return;
        RaiseClick(e);
        e.Handled = true;
    }

    private void RaiseClick(RoutedEventArgs e)
    {
        Click?.Invoke(this, e);

        var cmd = Command;
        var param = CommandParameter;

        if (cmd?.CanExecute(param) == true)
            cmd.Execute(param);
    }

    // 2) 可选：监听 Command 变化，处理 CanExecuteChanged（用于“禁用态”/伪类）
    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);

        if (change.Property == CommandProperty)
        {
            if (change.OldValue is ICommand oldCmd)
                oldCmd.CanExecuteChanged -= OnCanExecuteChanged;

            if (change.NewValue is ICommand newCmd)
                newCmd.CanExecuteChanged += OnCanExecuteChanged;

            UpdateEnabledFromCommand();
        }
        else if (change.Property == CommandParameterProperty)
        {
            UpdateEnabledFromCommand();
        }
    }

    private void OnCanExecuteChanged(object? sender, EventArgs e) => UpdateEnabledFromCommand();

    private void UpdateEnabledFromCommand()
    {
        // 你可以选择：
        // A) 直接用 IsEnabled 跟着 Command.CanExecute 走（类似 Button）
        // B) 或者只更新伪类 :disabled，让样式变灰，但不强制 IsEnabled
        var cmd = Command;
        var param = CommandParameter;

        if (cmd is null)
            return;

        IsEnabled = cmd.CanExecute(param);
    }
}