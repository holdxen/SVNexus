namespace SVNexus.Models;

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
        Visual
    }

    public enum LineEnding
    {
        Lf,
        Crlf
    }
    
    public Kind DifferenceKind { get; set; }

    public string? Content { get; set; } = string.Empty;
    
    public LineEnding Ending { get; set; } = LineEnding.Lf;
    
    public string? VisualText { get; set; }
    
    public string Text => Content ?? VisualText ?? string.Empty;
}