using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using SVNexus.Utils;

namespace SVNexus.Views;

public partial class DifferenceView : UserControl
{
    public DifferenceView()
    {
        InitializeComponent();
        OldTextEditor.TextArea.TextView.ScrollOffsetChanged += TextViewOnScrollOffsetChanged;
        // NewTextEditor.TextArea.TextView.ScrollOffsetChanged += TextViewOnScrollOffsetChanged;
    }

    private void TextViewOnScrollOffsetChanged(object? sender, EventArgs e)
    {
        NewTextEditor.ScrollToHorizontalOffset(OldTextEditor.HorizontalOffset);
        NewTextEditor.ScrollToVerticalOffset(OldTextEditor.VerticalOffset);
    }
}