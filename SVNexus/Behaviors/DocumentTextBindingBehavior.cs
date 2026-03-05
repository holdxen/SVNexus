using System;
using Avalonia;
using Avalonia.Xaml.Interactivity;
using AvaloniaEdit;

namespace SVNexus.Behaviors;

public class DocumentTextBindingBehavior : Behavior<TextEditor>
{
    private TextEditor? _textEditor;

    public static readonly StyledProperty<string> TextProperty =
        AvaloniaProperty.Register<DocumentTextBindingBehavior, string>(nameof(Text), defaultValue: string.Empty);

    public string Text
    {
        get => GetValue(TextProperty);
        set => SetValue(TextProperty, value);
    }

    protected override void OnAttached()
    {
        base.OnAttached();
        
        var textEditor = AssociatedObject;

        if (textEditor is null) return;
        _textEditor = textEditor;
        _textEditor.TextChanged += TextChanged;
        this.GetObservable(TextProperty).Subscribe(TextPropertyChanged);
    }

    protected override void OnDetaching()
    {
        base.OnDetaching();

        if (_textEditor != null)
        {
            _textEditor.TextChanged -= TextChanged;
        }
    }

    private void TextChanged(object? sender, EventArgs e)
    {
        if (_textEditor?.Document != null)
        {
            Text = _textEditor.Document.Text;
        }
    }

    private void TextPropertyChanged(string text)
    {
        if (_textEditor?.Document is not null)
        {
            var caretOffset = _textEditor.CaretOffset;   // 保留光标位置
            _textEditor.Document.Text = text;

            // 【关键修复】光标位置限制在新文本长度内，防止越界崩溃
            _textEditor.CaretOffset = Math.Min(caretOffset, _textEditor.Document.TextLength);
        }
    }
}