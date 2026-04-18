using Avalonia.Media;

namespace SVNexus.Models;

public class DifferenceColor
{
    public IBrush Background { get; set; } = Brushes.Transparent;
    public IBrush HighLight { get; set; } = Brushes.Transparent;
    public IBrush Mark { get; set; } = Brushes.Transparent;
}
