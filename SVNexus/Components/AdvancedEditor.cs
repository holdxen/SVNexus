using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Runtime.CompilerServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Media;
using Avalonia.Media.Immutable;
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

    public static readonly StyledProperty<bool> ChangeOnlyProperty = AvaloniaProperty.Register<AdvancedEditor, bool>(
        nameof(ChangeOnly));

    public bool ChangeOnly
    {
        get => GetValue(ChangeOnlyProperty);
        set => SetValue(ChangeOnlyProperty, value);
    }

    protected override Type StyleKeyOverride { get; } = typeof(TextEditor);
    
    protected readonly BackgroundRenderer LineBackgroundRenderer;// = new();
    
    protected readonly LineNumberRender LineNumberRenderer;// = new();
    
    
    static AdvancedEditor()
    {
        LinesProperty.Changed.AddClassHandler<AdvancedEditor, List<DifferenceLine>>(OnLinesPropertyChanged);
        ChangeOnlyProperty.Changed.AddClassHandler<AdvancedEditor, bool>(OnChangeOnlyPropertyChanged);
    }
    
    private static void OnLinesPropertyChanged(AdvancedEditor target,
        AvaloniaPropertyChangedEventArgs<List<DifferenceLine>> args)
    {
        target.AffectRenders();
    }

    private static void OnChangeOnlyPropertyChanged(AdvancedEditor target, AvaloniaPropertyChangedEventArgs<bool> args)
    {
        target.AffectRenders();
    }

    public AdvancedEditor()
    {

        LineNumberRenderer = new LineNumberRender(this);

        LineBackgroundRenderer = new BackgroundRenderer(this);

        ChangeOnly = true;
        
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


    private void AffectRenders()
    {
        // LineBackgroundRenderer.Lines = lines;
        // LineNumberRenderer.Lines = lines;
        Document.Text = string.Join("\n", Lines.Map(l => ChangeOnly ? l.Where(i => i.DifferenceKind != DifferenceLine.Kind.Unchanged) : l).Select(i => i.Text));
        LineNumberRenderer.InvalidateMeasure();
        LineNumberRenderer.InvalidateVisual();
    }

    private int RealIndex(int index)
    {
        if (!ChangeOnly)
        {
            return index;
        }
        var count = 0;
        for (var i = 0; i < Lines.Count; i++)
        {
            if (Lines[i].DifferenceKind is DifferenceLine.Kind.Unchanged)
            {
                continue;
            }
            if (index == count)
            {
                return i;
            }
            count++;
        }
        
        // throw new IndexOutOfRangeException();
        return -1;
    }


    protected class LineNumberRender(AdvancedEditor editor) : LineNumberMargin
    {
        
        // public List<DifferenceLine> Lines { get; set; } = [];
        
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

            var count = editor.Lines.Count(i => i.Content is not null).ToString().Length;
            
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

                if (editor.Lines.Count > 0)
                {
                    lineNumber = editor.RealIndex(lineNumber);
                }
                    
                if (lineNumber < 0 || lineNumber >= editor.Lines.Count) continue;
                
                var differenceLine = editor.Lines[lineNumber];
                    
                if (differenceLine.Content is null) continue;

                var number = editor.Lines.Take(lineNumber).Count(i => i.Content is not null) + 1;
                    
                var text = new FormattedText(
                    number.ToString(CultureInfo.CurrentCulture),
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
    

    protected class BackgroundRenderer(AdvancedEditor editor): IBackgroundRenderer
    {
        // public List<DifferenceLine> Lines { get; set; } = [];

        public KnownLayer Layer => KnownLayer.Background;
    
        public void Draw(TextView textView, DrawingContext drawingContext)
        {
            if (!textView.VisualLinesValid)
            {
                Logger.Info("The text view must be valid.");
                return;
            }
            var colorCollection = Application.Current?.ActualThemeVariant == ThemeVariant.Light ? DifferenceColorCollection.Light : DifferenceColorCollection
                .Dark;


            DifferenceLine.Kind? kind = null;
            var start = 0.0;
            // var index = 0;
        
            // foreach (var visualLine in textView.VisualLines)
            for (var i = 0; i < textView.VisualLines.Count; i++)
            {
                // index++;
                var visualLine = textView.VisualLines[i];
                var lineNumber = visualLine.FirstDocumentLine.LineNumber - 1;

                if (editor.Lines.Count > 0)
                {
                    lineNumber = editor.RealIndex(lineNumber);
                }
                
                if (lineNumber < 0 || lineNumber >= editor.Lines.Count)
                {
                    Logger.Error($"Line number {lineNumber} is out of range");
                    continue;
                }
                var line = editor.Lines[lineNumber];
                var y = visualLine.VisualTop - textView.VerticalOffset;
                if (kind is null)
                {
                    if (line.DifferenceKind is not DifferenceLine.Kind.Visual)
                    {
                        kind = line.DifferenceKind;
                        Logger.Info($"Set line to: {line.DifferenceKind}");
                        start = y;
                    }
                }
                else
                {
                    if (kind != line.DifferenceKind)
                    {
                        var rect = new Rect(0, start, textView.Bounds.Width, y - start);
                        
                        Logger.Info($"Draw background: {rect}, {colorCollection.BackgroundColor(kind.GetValueOrDefault())}");
                        Logger.Info($"Kind: {kind} {line.DifferenceKind}");
                        
                        drawingContext.DrawRectangle(colorCollection.BackgroundColor(kind.GetValueOrDefault()), null, rect);  
                        kind = line.DifferenceKind;
                        start = y;
                    }

                }

                if (i != textView.VisualLines.Count - 1 && lineNumber != editor.Lines.Count - 1) continue;
                {
                    var rect = new Rect(0, start, textView.Bounds.Width, y - start + visualLine.Height);
                    Logger.Info($"About to finish: {rect}, {colorCollection.BackgroundColor(kind.GetValueOrDefault())}");
                    drawingContext.DrawRectangle(colorCollection.BackgroundColor(line.DifferenceKind), null, rect);
                }
            }

        }
    }
    
}



// public class OldDifferenceEditor : AdvancedEditor
// {
//     static OldDifferenceEditor()
//     {
//         LinesProperty.Changed.AddClassHandler<OldDifferenceEditor, List<DifferenceLine>>(OnLinesPropertyChanged);
//     }
//
//     private static void OnLinesPropertyChanged(OldDifferenceEditor target,
//         AvaloniaPropertyChangedEventArgs<List<DifferenceLine>> args)
//     {
//         target.AffectRenders(args.NewValue.Value);
//     }
//
// }
//
// public class NewDifferenceEditor : AdvancedEditor
// {
//     static NewDifferenceEditor()
//     {
//         LinesProperty.Changed.AddClassHandler<NewDifferenceEditor, List<DifferenceLine>>(OnLinesPropertyChanged);
//     }
//
//     private static void OnLinesPropertyChanged(NewDifferenceEditor target,
//         AvaloniaPropertyChangedEventArgs<List<DifferenceLine>> args)
//     {
//         target.AffectRenders(args.NewValue.Value);
//     }
// }