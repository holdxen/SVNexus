using System;
using System.Collections.Generic;
using System.Globalization;
using System.Linq;
using System.Text;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Shapes;
using Avalonia.Input;
using Avalonia.Layout;
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
    
    private static void OnLinesPropertyChanged(AdvancedEditor target, AvaloniaPropertyChangedEventArgs<List<DifferenceLine>> args)
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
        
        TextArea.TextView.BackgroundRenderers.Add(LineBackgroundRenderer);
        
        ShowLineNumbers = false;
        
#if false
        var line = DottedLineMargin.Create();
        TextArea.LeftMargins.Insert(0, LineNumberRenderer);
        TextArea.LeftMargins.Insert(1, line);

        var foreground = this.GetBindingObservable(LineNumbersForegroundProperty);
        var margin = this.GetBindingObservable(LineNumbersMarginProperty);
        
        line.Bind(Shape.StrokeProperty, foreground);
        line.Bind(MarginProperty, margin);
        LineNumberRenderer.Bind(ForegroundProperty, foreground);
#endif
        
    }


    private void AffectRenders()
    {
        // LineBackgroundRenderer.Lines = lines;
        // LineNumberRenderer.Lines = lines;
        // Document.Text = string.Join("\n", Lines.Map(l => ChangeOnly ? l.Where(i => i.DifferenceKind != DifferenceLine.Kind.Unchanged) : l).Select(i => i.Text));

        if (ChangeOnly)
        {
            var lines = Lines.Where(i => i.DifferenceKind is not DifferenceLine.Kind.Unchanged).ToList();
            if (lines.Count == 0)
            {
                Document.Text = string.Empty;
                return;
            }

            lines[^1] = lines[^1] with { Ending = DifferenceLine.LineEnding.None };

            Document.Text = DifferenceLine.ToText(lines);
        }
        else
        {
            if (Lines.Count == 0)
            {
                Document.Text = string.Empty;
                return;
            }
            
            Document.Text = DifferenceLine.ToText(Lines);
        }
        
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
        
        public bool AlignLeft { get; set; }
        
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
                drawingContext.DrawText(text, new Point(AlignLeft ? 0 : renderSize.Width - text.Width, y - textView.VerticalOffset));
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

                // Logger.Info($"LineNumber {lineNumber} {GetHashCode()}");
                if (editor.Lines.Count > 0)
                {
                    lineNumber = editor.RealIndex(lineNumber);
                }
                // Logger.Info($"Translated lineNumber: {lineNumber} {GetHashCode()}");
                
                if (lineNumber < 0 || lineNumber >= editor.Lines.Count)
                {
                    // Logger.Error($"Line number {lineNumber} is out of range {GetHashCode()}");
                    continue;
                }
                var line = editor.Lines[lineNumber];
                var y = visualLine.VisualTop - textView.VerticalOffset;
                if (kind is null)
                {
                    if (line.DifferenceKind is not DifferenceLine.Kind.Visual)
                    {
                        kind = line.DifferenceKind;
                        // Logger.Info($"Set line to: {line.DifferenceKind} {GetHashCode()}");
                        start = y;
                    }
                }
                else
                {
                    if (kind != line.DifferenceKind)
                    {
                        var rect = new Rect(0, start, textView.Bounds.Width, y - start);
                        
                        // Logger.Info($"Draw background: {rect}, {colorCollection.BackgroundColor(kind.GetValueOrDefault())} {GetHashCode()}");
                        // Logger.Info($"Kind: {kind} {line.DifferenceKind}");
                        
                        drawingContext.DrawRectangle(colorCollection.BackgroundColor(kind.GetValueOrDefault()), null, rect);  
                        kind = line.DifferenceKind;
                        start = y;
                    }

                }

                if (i != textView.VisualLines.Count - 1 && lineNumber != editor.Lines.Count - 1) continue;
                {
                    var rect = new Rect(0, start, textView.Bounds.Width, y - start + visualLine.Height);
                    // Logger.Info($"About to finish: {rect}, {colorCollection.BackgroundColor(kind.GetValueOrDefault())} {GetHashCode()}");
                    drawingContext.DrawRectangle(colorCollection.BackgroundColor(line.DifferenceKind), null, rect);
                }
            }

        }
    }
    
}



public class OldDifferenceEditor : AdvancedEditor
{
    public OldDifferenceEditor()
    {
        var line = DottedLineMargin.Create();
        TextArea.LeftMargins.Insert(0, line);
        TextArea.LeftMargins.Insert(1, LineNumberRenderer);

        LineNumberRenderer.AlignLeft = true;

        var foreground = this.GetBindingObservable(LineNumbersForegroundProperty);
        var margin = this.GetBindingObservable(LineNumbersMarginProperty);
        
        line.Bind(Shape.StrokeProperty, foreground);
        line.Bind(MarginProperty, margin);
        LineNumberRenderer.Bind(ForegroundProperty, foreground);
    }
}

public class NewDifferenceEditor : AdvancedEditor
{
    public NewDifferenceEditor()
    {
        var line = DottedLineMargin.Create();
        TextArea.LeftMargins.Insert(0, LineNumberRenderer);
        TextArea.LeftMargins.Insert(1, line);

        var foreground = this.GetBindingObservable(LineNumbersForegroundProperty);
        var margin = this.GetBindingObservable(LineNumbersMarginProperty);
        
        line.Bind(Shape.StrokeProperty, foreground);
        line.Bind(MarginProperty, margin);
        LineNumberRenderer.Bind(ForegroundProperty, foreground);
    }
}

public class CombinedDifferenceEditor : TextEditor
{

    private record Line(int Index, bool? Old);
    
    protected class LineNumberRender(CombinedDifferenceEditor editor) : LineNumberMargin
    {
        // public List<DifferenceLine> Lines { get; set; } = [];
        
        // public bool AlignLeft { get; set; }
        
        public bool Old { get; set; }
        
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

            var count = editor.Lines.Count(i => i.Map(e => Old ? e.Item1 : e.Item2).Content is not null).ToString().Length;
            
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

            
            var simplifyLines = editor.SimplifyLines();

            if (textView is not { VisualLinesValid: true }) return;
            var foreground = GetValue(TextBlock.ForegroundProperty);
            foreach (var line in textView.VisualLines) {
                var lineNumber = line.FirstDocumentLine.LineNumber - 1;

                if (lineNumber < 0 || lineNumber >= simplifyLines.Count) continue;
                
                if (simplifyLines[lineNumber].Old.Map(e => e is not null && e != Old))
                {
                    continue;
                }
                
                lineNumber = simplifyLines[lineNumber].Index;
                    
                
                var differenceLine = editor.Lines[lineNumber];
                    
                if (differenceLine.Map(e => Old ? e.Item1 : e.Item2).Content is null) continue;

                var number = editor.Lines.Take(lineNumber).Count(i => i.Map(e => Old ? e.Item1 : e.Item2).Content is not null) + 1;
                    
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
    
    protected override Type StyleKeyOverride { get; } = typeof(TextEditor);

    public static readonly StyledProperty<List<Tuple<DifferenceLine, DifferenceLine>>> LinesProperty = AvaloniaProperty.Register<CombinedDifferenceEditor, List<Tuple<DifferenceLine, DifferenceLine>>>(
        nameof(Lines), defaultValue: []);

    public List<Tuple<DifferenceLine, DifferenceLine>> Lines
    {
        get => GetValue(LinesProperty);
        set => SetValue(LinesProperty, value);
    }

    public static readonly StyledProperty<bool> ChangeOnlyProperty = AvaloniaProperty.Register<CombinedDifferenceEditor, bool>(
        nameof(ChangeOnly));

    public bool ChangeOnly
    {
        get => GetValue(ChangeOnlyProperty);
        set => SetValue(ChangeOnlyProperty, value);
    }

    public CombinedDifferenceEditor()
    {
        TextArea.TextView.BackgroundRenderers.Add(new BackgroundRenderer(this));

        ShowLineNumbers = false;

        var newLineNumberRenderer = new LineNumberRender(this);
        var oldLineNumberRenderer = new LineNumberRender(this);
        
        var line = DottedLineMargin.Create();
        TextArea.LeftMargins.Insert(0, oldLineNumberRenderer);
        TextArea.LeftMargins.Insert(1, new Border()
        {
            Width = 3
        });
        TextArea.LeftMargins.Insert(2, newLineNumberRenderer);
        TextArea.LeftMargins.Insert(3, line);

        newLineNumberRenderer.Old = false;
        oldLineNumberRenderer.Old = true;

        var foreground = this.GetBindingObservable(LineNumbersForegroundProperty);
        var margin = this.GetBindingObservable(LineNumbersMarginProperty);
        
        line.Bind(Shape.StrokeProperty, foreground);
        line.Bind(MarginProperty, margin);
        newLineNumberRenderer.Bind(ForegroundProperty, foreground);
        oldLineNumberRenderer.Bind(ForegroundProperty, foreground);
    }

    protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
    {
        base.OnPropertyChanged(change);
        if (change.Property == LinesProperty || change.Property == ChangeOnlyProperty)
        {
            AffectRenders();
        }
    }

    private void AffectRenders()
    {

        var builder = new StringBuilder();

        var count = Lines.Count;
        for (var i = 0; i < count;)
        {
            // var oldLine = OldLines[i];
            // var newLine = NewLines[i];
            var (oldLine, newLine) = Lines[i];
            switch (new Tuple<DifferenceLine.Kind, DifferenceLine.Kind>(oldLine.DifferenceKind, newLine.DifferenceKind))
            {
                case (DifferenceLine.Kind.Visual, DifferenceLine.Kind.Visual):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Visual && l.Item2.DifferenceKind == DifferenceLine.Kind.Visual);

                    foreach (var line in lines)
                    {
                        builder.Append(line.Item1.VisualText);
                        builder.Append(line.Item1.EndingText());
                        i++;
                    }
                    break;
                }
                case (DifferenceLine.Kind.Unchanged, DifferenceLine.Kind.Unchanged):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Unchanged && l.Item2.DifferenceKind == DifferenceLine.Kind.Unchanged);

                    foreach (var line in lines)
                    {
                        if (!ChangeOnly)
                        {
                            builder.Append(line.Item1.Text);
                            builder.Append(line.Item1.EndingText());
                        }
                        i++;
                    }
                    break;
                }
                case (DifferenceLine.Kind.Add, DifferenceLine.Kind.Added):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Add && l.Item2.DifferenceKind == DifferenceLine.Kind.Added);

                    foreach (var line in lines)
                    {
                        builder.Append(line.Item2.Text);
                        builder.Append(line.Item2.EndingText());
                        i++;
                    }

                    break;
                }
                case (DifferenceLine.Kind.Remove, DifferenceLine.Kind.Removed):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Remove && l.Item2.DifferenceKind == DifferenceLine.Kind.Removed);

                    foreach (var line in lines)
                    {
                        builder.Append(line.Item1.Text);
                        builder.Append(line.Item1.EndingText());
                        i++;
                    }

                    break;
                }
                case (DifferenceLine.Kind.Modified, DifferenceLine.Kind.Modified):
                {
                    var newTextBuilder = new StringBuilder();
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Modified && l.Item2.DifferenceKind == DifferenceLine.Kind.Modified);

                    foreach (var line in lines)
                    {
                        Logger.Info($"Modified line: {line}");
                        i++;
                        if (line.Item1.Content is not null)
                        {
                            builder.Append(line.Item1.Content);
                            builder.Append(line.Item1.EndingText());
                        }


                        if (line.Item2.Content is null) continue;
                        
                        newTextBuilder.Append(line.Item2.Content);
                        newTextBuilder.Append(line.Item2.EndingText());
                    }

                
                    builder.Append(newTextBuilder);
                    break;
                }
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }

        Document.Text = builder.ToString();
    }

    private List<Line> SimplifyLines()
    {
        List<Line> simplifyLines = [];
        for (var i = 0; i < Lines.Count;)
        {
            var (oldLine, newLine) = Lines[i];
            switch (new Tuple<DifferenceLine.Kind, DifferenceLine.Kind>(oldLine.DifferenceKind, newLine.DifferenceKind))
            {
                case (DifferenceLine.Kind.Visual, DifferenceLine.Kind.Visual):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Visual && l.Item2.DifferenceKind == DifferenceLine.Kind.Visual);

                    var count = lines.Count();
                    
                    simplifyLines.AddRange(Enumerable.Range(i, count).Select(j => new Line(j, false)));
                    
                    i += count;
                    
                    break;
                }
                case (DifferenceLine.Kind.Unchanged, DifferenceLine.Kind.Unchanged):
                {
                    
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Unchanged && l.Item2.DifferenceKind == DifferenceLine.Kind.Unchanged);

                    var count = lines.Count();

                    if (!ChangeOnly)
                    {
                        simplifyLines.AddRange(Enumerable.Range(i, count).Select(j => new Line(j, null)));
                    }
                    
                    i += count;
                    
                    break;
                }
                case (DifferenceLine.Kind.Add, DifferenceLine.Kind.Added):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Add && l.Item2.DifferenceKind == DifferenceLine.Kind.Added);
                    
                    var count = lines.Count();
                    
                    simplifyLines.AddRange(Enumerable.Range(i, count).Select(j => new Line(j, false)));
                    
                    i += count;

                    break;
                }
                case (DifferenceLine.Kind.Remove, DifferenceLine.Kind.Removed):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Remove && l.Item2.DifferenceKind == DifferenceLine.Kind.Removed);
                    
                    var count = lines.Count();
                    
                    simplifyLines.AddRange(Enumerable.Range(i, count).Select(j => new Line(j, true)));
    
                    i += count;

                    break;
                }
                case (DifferenceLine.Kind.Modified, DifferenceLine.Kind.Modified):
                {
                    var lines = Lines.Skip(i).TakeWhile(l => l.Item1.DifferenceKind == DifferenceLine.Kind.Modified && l.Item2.DifferenceKind == DifferenceLine.Kind.Modified).ToList();

                    var count = lines.Count(j => j.Item1.Content is not null);
                    simplifyLines.AddRange(Enumerable.Range(i, count).Select(j => new Line(j, true)));
                    
                    count = lines.Count(j => j.Item2.Content is not null);
                    simplifyLines.AddRange(Enumerable.Range(i, count).Select(j => new Line(j, false)));
                    i += lines.Count;


                    break;
                }
                default:
                    throw new ArgumentOutOfRangeException();
            }
        }
        return simplifyLines;
    }


    private class BackgroundRenderer(CombinedDifferenceEditor editor) : IBackgroundRenderer
    {
        public void Draw(TextView textView, DrawingContext drawingContext)
        {
            // if (editor.NewLines.Count != editor.OldLines.Count)
            // {
            //     Logger.Info($"Invalid text lines: new={editor.NewLines.Count}, old={editor.OldLines.Count}");
            //     return;
            // }
            //
            //
            // for (var i = 0; i < editor.OldLines.Count; i++)
            // {
            //     
            // }
            
            var colorCollection = Application.Current?.ActualThemeVariant == ThemeVariant.Light ? DifferenceColorCollection.Light : DifferenceColorCollection
                .Dark;

            if (!textView.VisualLinesValid)
            {
                return;
            }

            var simplifyLines = editor.SimplifyLines();
            
            DifferenceLine.Kind? kind = null;
            Line? whichLine = null;
            var start = 0.0;

            // foreach (var visualLine in textView.VisualLines)
            for (var i = 0; i < textView.VisualLines.Count; i++)
            {
                var visualLine = textView.VisualLines[i];
                
                var lineNumber = visualLine.FirstDocumentLine.LineNumber - 1;

                if (lineNumber < 0 || lineNumber >= simplifyLines.Count)
                {
                    continue;
                }
                
                var simplifyLine = simplifyLines[lineNumber];

                if (simplifyLine.Index < 0 || simplifyLine.Index >= editor.Lines.Count)
                {
                    Logger.Error($"Invalid index: index={simplifyLine.Index}, count={editor.Lines.Count}");
                }
                var line = editor.Lines[simplifyLine.Index].Map(j => simplifyLine.Old ?? false ? j.Item1 : j.Item2);
               
 
                var y = visualLine.VisualTop - textView.VerticalOffset;
                if (kind is null || whichLine is null)
                {
                    if (line.DifferenceKind is not DifferenceLine.Kind.Visual)
                    {
                        kind = line.DifferenceKind;
                        whichLine = simplifyLine;
                        // colorKind = line.DifferenceKind;
                        // if (colorKind == DifferenceLine.Kind.Modified)
                        // {
                        //     colorKind = simplifyLine.Old ? DifferenceLine.Kind.Removed  : DifferenceLine.Kind.Added;
                        // }
                        // Logger.Info($"Set line to: {line.DifferenceKind} {GetHashCode()}");
                        start = y;
                    }
                }
                else
                {
                    if (kind != line.DifferenceKind || whichLine.Old != simplifyLine.Old)
                    {
                        var rect = new Rect(0, start, textView.Bounds.Width, y - start);
                        
                        // Logger.Info($"Draw background: {rect}, {colorCollection.BackgroundColor(kind.GetValueOrDefault())} {GetHashCode()}");
                        // Logger.Info($"Kind: {kind} {line.DifferenceKind}");
                        var currentKind = kind;
                        if (currentKind == DifferenceLine.Kind.Modified)
                        {
                            currentKind = whichLine.Old ?? false ? DifferenceLine.Kind.Removed : DifferenceLine.Kind.Added;
                        }

                        drawingContext.DrawRectangle(colorCollection.BackgroundColor(currentKind.GetValueOrDefault()), null, rect);  
                        kind = line.DifferenceKind;
                        whichLine = simplifyLine;
                        start = y;
                    }

                }

                if (i != textView.VisualLines.Count - 1 && lineNumber != simplifyLines.Count - 1) continue;
                {
                    var rect = new Rect(0, start, textView.Bounds.Width, y - start + visualLine.Height);
                    // Logger.Info($"About to finish: {rect}, {colorCollection.BackgroundColor(kind.GetValueOrDefault())} {GetHashCode()}");
                    var currentKind = line.DifferenceKind;
                    if (currentKind == DifferenceLine.Kind.Modified)
                    {
                        currentKind = simplifyLine.Old ?? false ? DifferenceLine.Kind.Removed : DifferenceLine.Kind.Added;
                    }
                    drawingContext.DrawRectangle(colorCollection.BackgroundColor(currentKind), null, rect);
                }

                
            }
        }

        public KnownLayer Layer => KnownLayer.Background;
    }
}