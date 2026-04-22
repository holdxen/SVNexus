using System;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.VisualTree;
using SVNexus.Utils;

namespace SVNexus.Views;

public partial class DifferenceView : UserControl
{
    public DifferenceView()
    {
        InitializeComponent();
        OldTextEditor.TextArea.TextView.ScrollOffsetChanged += OldTextViewOnScrollOffsetChanged;
        NewTextEditor.TextArea.TextView.ScrollOffsetChanged += NewTextViewOnScrollOffsetChanged;
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