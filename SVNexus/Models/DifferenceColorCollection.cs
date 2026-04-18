using Avalonia.Media;
using Avalonia.Media.Immutable;

namespace SVNexus.Models;

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

