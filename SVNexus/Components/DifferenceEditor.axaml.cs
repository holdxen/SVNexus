using System.Collections.Generic;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Markup.Xaml;
using Avalonia.Media;
using Avalonia.Media.Immutable;
using AvaloniaEdit;
using AvaloniaEdit.Document;
using AvaloniaEdit.Editing;
using AvaloniaEdit.Rendering;

namespace SVNexus.Components;

public enum DiffKind { Unchanged, Added, Removed, Modified, Placeholder }

// 每一行“对齐后的显示行”
public sealed record DiffRow(
    string? LeftText,   // null => 左侧占位
    string? RightText,  // null => 右侧占位
    DiffKind Kind       // 这行整体属于哪种 diff（按你规则拆到左右也可以）
);

public sealed class DiffDisplayMap
{
    public required Dictionary<int, DiffKind> LineKind { get; init; } // key: display lineNumber(1-based)
    
    public static (TextDocument left, TextDocument right, DiffDisplayMap leftMap, DiffDisplayMap rightMap)
        BuildAlignedDocuments(IReadOnlyList<DiffRow> rows)
    {
        var leftLines = new List<string>(rows.Count);
        var rightLines = new List<string>(rows.Count);

        var leftMap = new DiffDisplayMap { LineKind = new Dictionary<int, DiffKind>() };
        var rightMap = new DiffDisplayMap { LineKind = new Dictionary<int, DiffKind>() };

        int line = 0;
        foreach (var r in rows)
        {
            line++;

            leftLines.Add(r.LeftText ?? "");
            rightLines.Add(r.RightText ?? "");

            leftMap.LineKind[line]  = r.LeftText  is null ? DiffKind.Placeholder : r.Kind;
            rightMap.LineKind[line] = r.RightText is null ? DiffKind.Placeholder : r.Kind;
        }

        return (new TextDocument(string.Join("\n", leftLines)),
            new TextDocument(string.Join("\n", rightLines)),
            leftMap, rightMap);
    }
}


public enum EditorRole
{
    Original,
    Modified,
}


public class LineNumber : AbstractMargin
{
    
}

public sealed class DifferenceLineBackgroundRenderer(IReadOnlyDictionary<int, DiffKind> lineKind) : IBackgroundRenderer
{
    
    
    // 画在背景层（在 Selection 下面）
    public KnownLayer Layer => KnownLayer.Background;

    // 你可以换成主题 Brush / 资源引用
    private static readonly IBrush AddedBrush =
        new ImmutableSolidColorBrush(Color.FromArgb(40, 0, 200, 0));
    private static readonly IBrush RemovedBrush =
        new ImmutableSolidColorBrush(Color.FromArgb(40, 220, 0, 0));
    private static readonly IBrush ModifiedBrush =
        new ImmutableSolidColorBrush(Color.FromArgb(40, 255, 200, 0));
    private static readonly IBrush PlaceholderBrush =
        new ImmutableSolidColorBrush(Color.FromArgb(25, 120, 120, 120));

    public void Draw(TextView textView, DrawingContext drawingContext)
    {
        if (!textView.VisualLinesValid)
        {
            return;
        }

        foreach (var vl in textView.VisualLines)
        {
            var lineNo = vl.FirstDocumentLine.LineNumber;

            if (!lineKind.TryGetValue(lineNo, out var kind))
                continue;

            var brush = kind switch
            {
                DiffKind.Added => AddedBrush,
                DiffKind.Removed => RemovedBrush,
                DiffKind.Modified => ModifiedBrush,
                DiffKind.Placeholder => PlaceholderBrush,
                _ => null
            };

            if (brush is null) continue;

            // VisualTop 是“相对文档”的 Y，减去 VerticalOffset => 当前视口内的 Y
            var y = vl.VisualTop - textView.VerticalOffset;

            // 覆盖整行（包含行尾空白）
            var rect = new Rect(0, y, textView.Bounds.Width, vl.Height);
            drawingContext.DrawRectangle(brush, null, rect);
        }
    }
}

public partial class DifferenceEditor : UserControl
{
    public DifferenceEditor()
    {
        InitializeComponent();
        var editor = this.FindControl<TextEditor>("LeftText")!;
    }
}