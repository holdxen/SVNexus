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
    }
    public Kind DifferenceKind { get; set; }

    public string Content { get; set; } = string.Empty;
}