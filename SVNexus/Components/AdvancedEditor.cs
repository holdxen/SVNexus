using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Runtime.CompilerServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Media;
using Avalonia.Styling;
using AvaloniaEdit;
using AvaloniaEdit.Editing;
using AvaloniaEdit.Rendering;
using AvaloniaEdit.Utils;
using SVNexus.Extension;
using SVNexus.Models;
using SVNexus.Utils;

namespace SVNexus.Components;

public class AdvancedEditor: TextEditor
{
    public static readonly StyledProperty<List<DifferenceLine>> LinesProperty = AvaloniaProperty.Register<AdvancedEditor, List<DifferenceLine>>(
        nameof(Lines), defaultValue: []);

    public List<DifferenceLine> Lines
    {
        get => GetValue(LinesProperty);
        set => SetValue(LinesProperty, value);
    }

    protected override Type StyleKeyOverride { get; } = typeof(TextEditor);
    
    protected readonly BackgroundRenderer LineBackgroundRenderer = new();
    
    protected readonly LineNumberRender LineNumberRenderer = new();
    

    public AdvancedEditor()
    {
        TextArea.TextView.BackgroundRenderers.Add(LineBackgroundRenderer);
        
        var line = DottedLineMargin.Create();
        TextArea.LeftMargins.Insert(0, LineNumberRenderer);
        TextArea.LeftMargins.Insert(1, line);

        var foreground = this.GetBindingObservable(LineNumbersForegroundProperty);
        var margin = this.GetBindingObservable(LineNumbersMarginProperty);
        
        line.Bind(Shape.StrokeProperty, foreground);
        line.Bind(MarginProperty, margin);
        LineNumberRenderer.Bind(ForegroundProperty, foreground);
        
        ShowLineNumbers = false;
    }

    protected void AffectRenders(List<DifferenceLine> lines)
    {
        LineBackgroundRenderer.Lines = lines;
        LineNumberRenderer.Lines = lines;
        Document.Text = string.Join(Environment.NewLine, lines.Select(i => i.Content ?? string.Empty));
        LineNumberRenderer.InvalidateMeasure();
        LineNumberRenderer.InvalidateVisual();

    }


    public class LineNumberRender : LineNumberMargin
    {
        
        public List<DifferenceLine> Lines { get; set; } = [];
        
        protected override Size MeasureOverride(Size availableSize)
        {
            Typeface = this.CreateTypeface();
            EmSize = GetValue(TextBlock.FontSizeProperty);

            // var text = TextFormatterFactory.CreateFormattedText(
            //     this,
            //     new string('9', MaxLineNumberLength),
            //     Typeface,
            //     EmSize,
            //     GetValue(TextBlock.ForegroundProperty)
            // );

            var count = Lines.Count(i => i.Content is not null).ToString().Length;
            
            var text = new FormattedText(
                new string('9', count),
                CultureInfo.CurrentCulture,
                FlowDirection.LeftToRight,
                Typeface,
                EmSize,
                GetValue(TextBlock.ForegroundProperty)
            );
            return new Size(text.Width, 0);
        }
        
        public override void Render(DrawingContext drawingContext)
        {
            var textView = TextView;
            var renderSize = Bounds.Size;


            if (textView is not { VisualLinesValid: true }) return;
            var foreground = GetValue(TextBlock.ForegroundProperty);
            foreach (var line in textView.VisualLines) {
                var lineNumber = line.FirstDocumentLine.LineNumber - 1;
                // var text = TextFormatterFactory.CreateFormattedText(
                //     this,
                //     lineNumber.ToString(CultureInfo.CurrentCulture),
                //     Typeface, EmSize, foreground
                // );
                    
                if (lineNumber < 0 || lineNumber >= Lines.Count) continue;
                var differenceLine = Lines[lineNumber];
                    
                if (differenceLine.Content is null) continue;


                // var realIndex = Lines.RealIndex(lineNumber);

                var index = Lines.Take(lineNumber).Count(i => i.Content is not null);
                    
                // var c = Lines.RealIndex(lineNumber);
                    
                // Logger.Info($"Render: index={index}, c={c}");
                    
                //
                //
                // var index = Lines.RealIndex(lineNumber);
                //
                // Logger.Info($"Render number: lineNumber={lineNumber}, index={index}");
                    
                    
                var text = new FormattedText(
                    index.ToString(CultureInfo.CurrentCulture),
                    CultureInfo.CurrentCulture,
                    FlowDirection.LeftToRight,
                    Typeface,
                    EmSize,
                    foreground
                );
                var y = line.GetTextLineVisualYPosition(line.TextLines[0], VisualYPosition.TextTop);
                drawingContext.DrawText(text, new Point(renderSize.Width - text.Width, y - textView.VerticalOffset));
            }
        }
    }
    

    protected class BackgroundRenderer: IBackgroundRenderer
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
        target.AffectRenders(args.NewValue.Value);
        // var value = args.NewValue.Value;
        //
        // var lines = value.Select(i => i.Content ?? string.Empty).ToList();
        //
        //
        // target.LineBackgroundRenderer.Lines = value;
        // target.Document.Text = string.Join(Environment.NewLine, lines);
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
        target.AffectRenders(args.NewValue.Value);
    }


}