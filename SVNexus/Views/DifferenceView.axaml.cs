using System;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.VisualTree;
using AvaloniaEdit;

namespace SVNexus.Views;


public sealed class DiffScrollSynchronizer
{
    private ScrollViewer? _leftSv;
    private ScrollViewer? _rightSv;
    private bool _syncing;

    public DiffScrollSynchronizer(TextEditor left, TextEditor right)
    {
        Hook(left, isLeft: true);
        Hook(right, isLeft: false);
    }

    private void Hook(TextEditor editor, bool isLeft)
    {
        // 如果模板已经应用过，直接拿；否则等 TemplateApplied
        var sv = editor.FindDescendantOfType<ScrollViewer>();
        if (sv != null)
        {
            Bind(sv, isLeft);
        }
        else
        {
            editor.TemplateApplied += (_, e) =>
            {
                var found = e.NameScope.Find<ScrollViewer>("PART_ScrollViewer")
                            ?? editor.FindDescendantOfType<ScrollViewer>();
                if (found != null) Bind(found, isLeft);
            };
        }
    }

    private void Bind(ScrollViewer sv, bool isLeft)
    {
        if (isLeft) _leftSv = sv; else _rightSv = sv;

        // ScrollChanged 在 Offset / Extent / Viewport 任意变化时触发
        sv.ScrollChanged += (_, _) =>
        {
            if (isLeft) SyncTo(_leftSv, _rightSv);
            else        SyncTo(_rightSv, _leftSv);
        };
    }

    private void SyncTo(ScrollViewer? source, ScrollViewer? target)
    {
        if (_syncing || source is null || target is null) return;

        _syncing = true;
        try
        {
            var srcMaxX = source.Extent.Width  - source.Viewport.Width;
            var srcMaxY = source.Extent.Height - source.Viewport.Height;
            var dstMaxX = target.Extent.Width  - target.Viewport.Width;
            var dstMaxY = target.Extent.Height - target.Viewport.Height;

            // 按比例换算，extent==viewport 时归零，避免除 0
            var px = srcMaxX > 0 ? source.Offset.X / srcMaxX : 0;
            var py = srcMaxY > 0 ? source.Offset.Y / srcMaxY : 0;

            var newX = dstMaxX > 0 ? dstMaxX * px : 0;
            var newY = dstMaxY > 0 ? dstMaxY * py : 0;

            // 只有真正变化时才赋值，进一步减少回环
            if (Math.Abs(target.Offset.X - newX) > 0.5 ||
                Math.Abs(target.Offset.Y - newY) > 0.5)
            {
                target.Offset = new Vector(newX, newY);
            }
        }
        finally
        {
            _syncing = false;
        }
    }
}

public partial class DifferenceView : UserControl
{
    public DifferenceView()
    {
        InitializeComponent();
        // OldTextEditor.TextArea.TextView.ScrollOffsetChanged += OldTextViewOnScrollOffsetChanged;
        // NewTextEditor.TextArea.TextView.ScrollOffsetChanged += NewTextViewOnScrollOffsetChanged;
        _ = new DiffScrollSynchronizer(OldTextEditor, NewTextEditor);
    }
    
    private void NewTextViewOnScrollOffsetChanged(object? sender, EventArgs e)
    {
        // NewTextEditor.ScrollToHorizontalOffset(OldTextEditor.HorizontalOffset);
        // NewTextEditor.ScrollToVerticalOffset(OldTextEditor.VerticalOffset);
        
        var sv = OldTextEditor.GetVisualDescendants()
            .OfType<ScrollViewer>()
            .FirstOrDefault();

        OldTextEditor.TextArea.TextView.ScrollOffsetChanged -= OldTextViewOnScrollOffsetChanged;
        sv?.Offset = new Vector(NewTextEditor.HorizontalOffset, NewTextEditor.VerticalOffset);
        OldTextEditor.TextArea.TextView.ScrollOffsetChanged += OldTextViewOnScrollOffsetChanged;
    }

    private void OldTextViewOnScrollOffsetChanged(object? sender, EventArgs e)
    {
        // NewTextEditor.ScrollToHorizontalOffset(OldTextEditor.HorizontalOffset);
        // NewTextEditor.ScrollToVerticalOffset(OldTextEditor.VerticalOffset);
        
        var sv = NewTextEditor.GetVisualDescendants()
            .OfType<ScrollViewer>()
            .FirstOrDefault();
        
        NewTextEditor.TextArea.TextView.ScrollOffsetChanged -= NewTextViewOnScrollOffsetChanged;
        
        sv?.Offset = new Vector(OldTextEditor.HorizontalOffset, OldTextEditor.VerticalOffset);
        NewTextEditor.TextArea.TextView.ScrollOffsetChanged += NewTextViewOnScrollOffsetChanged;
    }
}