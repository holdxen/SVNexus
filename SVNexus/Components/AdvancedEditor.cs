using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Media;
using Avalonia.Styling;
using AvaloniaEdit;
using AvaloniaEdit.Rendering;
using SVNexus.Models;

namespace SVNexus.Components;

public class AdvancedEditor: TextEditor
{
    public static readonly StyledProperty<List<DifferenceLine>> LinesProperty = AvaloniaProperty.Register<AdvancedEditor, List<DifferenceLine>>(
        nameof(Lines));

    public List<DifferenceLine> Lines
    {
        get => GetValue(LinesProperty);
        set => SetValue(LinesProperty, value);
    }

    protected override Type StyleKeyOverride { get; } = typeof(TextEditor);
    
    protected readonly BackgroundRenderer LineBackgroundRenderer = new();

    public AdvancedEditor()
    {
        TextArea.TextView.BackgroundRenderers.Add(LineBackgroundRenderer);
    }
    
    public class BackgroundRenderer: IBackgroundRenderer
    {
        public List<DifferenceLine> Lines { get; set; } = [];

        public KnownLayer Layer => KnownLayer.Background;
    
        public void Draw(TextView textView, DrawingContext drawingContext)
        {
            if (!textView.VisualLinesValid)
            {
                return;
            }
            var colorCollection = Application.Current?.ActualThemeVariant == ThemeVariant.Light ? DifferenceColorCollection.Light : DifferenceColorCollection
                .Dark;
        
        
            foreach (var visualLine in textView.VisualLines)
            {
                var lineNumber = visualLine.FirstDocumentLine.LineNumber - 1;
                if (lineNumber < 0 || lineNumber >= Lines.Count) continue;
                IBrush? brush = null;
                var line = Lines[lineNumber];
                switch (line.DifferenceKind)
                {
                    case DifferenceLine.Kind.Modified:
                        brush = colorCollection.Modified.Background;
                        break;
                    case DifferenceLine.Kind.Unchanged:
                        break;
                    case DifferenceLine.Kind.Add:
                        brush = colorCollection.Placeholder.Background;
                        break;
                    case DifferenceLine.Kind.Added:
                        brush = colorCollection.Added.Background;
                        break;
                    case DifferenceLine.Kind.Remove:
                        brush = colorCollection.Removed.Background;
                        break;
                    case DifferenceLine.Kind.Removed:
                        brush = colorCollection.Placeholder.Background;
                        break;
                    default:
                        throw new ArgumentOutOfRangeException(nameof(line.DifferenceKind), line.DifferenceKind, "Invalid difference kind");
                }

                if (brush is null) continue;
                var y = visualLine.VisualTop - textView.VerticalOffset;

                // 覆盖整行（包含行尾空白）
                var rect = new Rect(0, y, textView.Bounds.Width, visualLine.Height);
                drawingContext.DrawRectangle(brush, null, rect);
            
            }

        }
    }
    
}



public class OldDifferenceEditor : AdvancedEditor
{
    static OldDifferenceEditor()
    {
        LinesProperty.Changed.AddClassHandler<OldDifferenceEditor, List<DifferenceLine>>(OnLinesPropertyChanged);
    }

    private static void OnLinesPropertyChanged(OldDifferenceEditor target,
        AvaloniaPropertyChangedEventArgs<List<DifferenceLine>> args)
    {
        var value = args.NewValue.Value;

        List<string> lines = [];

        foreach (var line in value)
        {
            switch (line.DifferenceKind)
            {
                case DifferenceLine.Kind.Unchanged:
                    lines.Add(line.Content);
                    break;
                case DifferenceLine.Kind.Add:
                    lines.Add("");
                    break;
                case DifferenceLine.Kind.Remove:
                case DifferenceLine.Kind.Modified:
                    lines.Add(line.Content);
                    break;
                case DifferenceLine.Kind.Added:
                case DifferenceLine.Kind.Removed:
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }
        target.LineBackgroundRenderer.Lines = value;
        target.Document.Text = string.Join(Environment.NewLine, lines);
    }

}

public class NewDifferenceEditor : AdvancedEditor
{
    static NewDifferenceEditor()
    {
        LinesProperty.Changed.AddClassHandler<NewDifferenceEditor, List<DifferenceLine>>(OnLinesPropertyChanged);
    }

    private static void OnLinesPropertyChanged(NewDifferenceEditor target,
        AvaloniaPropertyChangedEventArgs<List<DifferenceLine>> args)
    {
        var value = args.NewValue.Value;

        List<string> lines = [];

        foreach (var line in value)
        {
            switch (line.DifferenceKind)
            {
                case DifferenceLine.Kind.Unchanged:
                case DifferenceLine.Kind.Added:
                    lines.Add(line.Content);
                    break;
                case DifferenceLine.Kind.Removed:
                    lines.Add("");
                    break;
                case DifferenceLine.Kind.Modified:
                    lines.Add(line.Content);
                    break;
                case DifferenceLine.Kind.Add:
                case DifferenceLine.Kind.Remove:
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }
        
        target.LineBackgroundRenderer.Lines = value;
        target.Document.Text = string.Join(Environment.NewLine, lines);
    }


}