using System;
using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Media;
using Avalonia.Media.Immutable;
using Avalonia.Styling;
using AvaloniaEdit;
using AvaloniaEdit.Rendering;

namespace SVNexus.Components;

public class Difference
{
    public List<DifferenceLine> Original { get; set; } = [];
    
    public List<DifferenceLine> Modified { get; set; } = [];
    
}

public class DifferenceLine
{
    public enum Kind 
    {
        Unchanged,
        Added,
        Add,
        Removed,
        Remove,
        Modified,
    }
    public Kind DifferenceKind { get; set; }

    public string Content { get; set; } = string.Empty;
}

public class DifferenceColor
{
    public IBrush Background { get; set; } = Brushes.Transparent;
    public IBrush HighLight { get; set; } = Brushes.Transparent;
    public IBrush Mark { get; set; } = Brushes.Transparent;

}

public class DifferenceColorCollection
{
    public DifferenceColor Added { get; set; } = new();
    public DifferenceColor Removed { get; set; } = new();
    public DifferenceColor Modified { get; set; } = new();
    public DifferenceColor Placeholder { get; set; } = new();
    
    
    public static readonly DifferenceColorCollection Light = new()
    {
        Added = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#EAFBF1")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#C7EFD8")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#2DA44E")),
        },
        Removed = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#FFF1F0")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#FFD6D2")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#D1242F")),
        },
        Modified = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#FFF8E1")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#FFE7A3")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#D4A72C")),
        },
        Placeholder = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#EEF6FF")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#D6E9FF")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#1F6FEB")),
        },
    };
    public static readonly DifferenceColorCollection Dark = new()
    {
        Added = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#0F2419")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#123222")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#3FB950")),
        },
        Removed = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#2A1215")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#472326")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#F85149")),
        },
        Modified = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#2B230A")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#4A3A0A")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#E3B341")),
        },
        Placeholder = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#0D2238")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#14304A")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#58A6FF")),
        },
    };
}

public class DifferenceLineBackgroundRenderer: IBackgroundRenderer
{

    public List<DifferenceLine> Lines { get; set; } = [];
    

    public KnownLayer Layer => KnownLayer.Background;
    
    
    // private static readonly IBrush AddedBrush =
    //     new ImmutableSolidColorBrush(Color.FromArgb(40, 0, 200, 0));
    // private static readonly IBrush RemovedBrush =
    //     new ImmutableSolidColorBrush(Color.FromArgb(40, 220, 0, 0));
    // private static readonly IBrush ModifiedBrush =
    //     new ImmutableSolidColorBrush(Color.FromArgb(40, 255, 200, 0));
    // private static readonly IBrush PlaceholderBrush =
    //     new ImmutableSolidColorBrush(Color.FromArgb(25, 120, 120, 120));
    
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

// public enum DiffKind { Unchanged, Added, Removed, Modified, Placeholder }
//
// // 每一行“对齐后的显示行”
// public sealed record DiffRow(
//     string? LeftText,   // null => 左侧占位
//     string? RightText,  // null => 右侧占位
//     DiffKind Kind       // 这行整体属于哪种 diff（按你规则拆到左右也可以）
// );
//
// public sealed class DiffDisplayMap
// {
//     public required Dictionary<int, DiffKind> LineKind { get; init; } // key: display lineNumber(1-based)
//     
//     public static (TextDocument left, TextDocument right, DiffDisplayMap leftMap, DiffDisplayMap rightMap)
//         BuildAlignedDocuments(IReadOnlyList<DiffRow> rows)
//     {
//         var leftLines = new List<string>(rows.Count);
//         var rightLines = new List<string>(rows.Count);
//
//         var leftMap = new DiffDisplayMap { LineKind = new Dictionary<int, DiffKind>() };
//         var rightMap = new DiffDisplayMap { LineKind = new Dictionary<int, DiffKind>() };
//
//         int line = 0;
//         foreach (var r in rows)
//         {
//             line++;
//
//             leftLines.Add(r.LeftText ?? "");
//             rightLines.Add(r.RightText ?? "");
//
//             leftMap.LineKind[line]  = r.LeftText  is null ? DiffKind.Placeholder : r.Kind;
//             rightMap.LineKind[line] = r.RightText is null ? DiffKind.Placeholder : r.Kind;
//         }
//
//         return (new TextDocument(string.Join("\n", leftLines)),
//             new TextDocument(string.Join("\n", rightLines)),
//             leftMap, rightMap);
//     }
// }
//
//
// public enum EditorRole
// {
//     Original,
//     Modified,
// }
//
//
// public class LineNumber : AbstractMargin
// {
//     
// }
//
// public sealed class DifferenceLineBackgroundRenderer(IReadOnlyDictionary<int, DiffKind> lineKind) : IBackgroundRenderer
// {
//     
//     
//     // 画在背景层（在 Selection 下面）
//     public KnownLayer Layer => KnownLayer.Background;
//
//     // 你可以换成主题 Brush / 资源引用
//     private static readonly IBrush AddedBrush =
//         new ImmutableSolidColorBrush(Color.FromArgb(40, 0, 200, 0));
//     private static readonly IBrush RemovedBrush =
//         new ImmutableSolidColorBrush(Color.FromArgb(40, 220, 0, 0));
//     private static readonly IBrush ModifiedBrush =
//         new ImmutableSolidColorBrush(Color.FromArgb(40, 255, 200, 0));
//     private static readonly IBrush PlaceholderBrush =
//         new ImmutableSolidColorBrush(Color.FromArgb(25, 120, 120, 120));
//
//     public void Draw(TextView textView, DrawingContext drawingContext)
//     {
//         if (!textView.VisualLinesValid)
//         {
//             return;
//         }
//
//         foreach (var vl in textView.VisualLines)
//         {
//             var lineNo = vl.FirstDocumentLine.LineNumber;
//
//             if (!lineKind.TryGetValue(lineNo, out var kind))
//                 continue;
//
//             var brush = kind switch
//             {
//                 DiffKind.Added => AddedBrush,
//                 DiffKind.Removed => RemovedBrush,
//                 DiffKind.Modified => ModifiedBrush,
//                 DiffKind.Placeholder => PlaceholderBrush,
//                 _ => null
//             };
//
//             if (brush is null) continue;
//
//             // VisualTop 是“相对文档”的 Y，减去 VerticalOffset => 当前视口内的 Y
//             var y = vl.VisualTop - textView.VerticalOffset;
//
//             // 覆盖整行（包含行尾空白）
//             var rect = new Rect(0, y, textView.Bounds.Width, vl.Height);
//             drawingContext.DrawRectangle(brush, null, rect);
//         }
//     }
// }

public partial class DifferenceEditor : UserControl
{

    public static readonly StyledProperty<Difference> DifferenceProperty = AvaloniaProperty.Register<DifferenceEditor, Difference>(
        nameof(Difference), defaultValue: new Difference());

    public Difference Difference
    {
        get => GetValue(DifferenceProperty);
        set => SetValue(DifferenceProperty, value);
    }
    
    private readonly TextEditor _leftEditor;
    private readonly DifferenceLineBackgroundRenderer _leftEditorBackgroundRenderer = new();
    
    private readonly TextEditor _rightEditor;
    private readonly DifferenceLineBackgroundRenderer _rightEditorBackgroundRenderer = new();
    
    public DifferenceEditor()
    {
        InitializeComponent();
        _leftEditor = this.FindControl<TextEditor>("LeftEditor")!;
        _rightEditor = this.FindControl<TextEditor>("RightEditor")!;
        
        
        _leftEditor.TextArea.TextView.BackgroundRenderers.Add(_leftEditorBackgroundRenderer);
        _rightEditor.TextArea.TextView.BackgroundRenderers.Add(_rightEditorBackgroundRenderer);
        
    }

    static DifferenceEditor()
    {
        DifferenceProperty.Changed.AddClassHandler<DifferenceEditor, Difference>(OnDifferencePropertyChanged);
    }

    private static void OnDifferencePropertyChanged(DifferenceEditor target, AvaloniaPropertyChangedEventArgs<Difference> args)
    {
        var value = args.NewValue.Value;

        List<string> lines = [];

        foreach (var line in value.Original)
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
        
        target._leftEditorBackgroundRenderer.Lines = value.Original;
        target._leftEditor.Document.Text = string.Join(Environment.NewLine, lines);
        
        
        lines.Clear();

        foreach (var line in value.Modified)
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
        
        target._rightEditorBackgroundRenderer.Lines = value.Modified;
        target._rightEditor.Document.Text = string.Join(Environment.NewLine, lines);
        
        
        
    }
}