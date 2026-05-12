using System;
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
    // public static readonly DifferenceColorCollection Dark = new()
    // {
    //     Added = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#0F2419")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#123222")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#3FB950")),
    //     },
    //     Removed = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#2A1215")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#472326")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#F85149")),
    //     },
    //     Modified = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#2B230A")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#4A3A0A")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#E3B341")),
    //     },
    //     Placeholder = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#0D2238")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#14304A")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#58A6FF")),
    //     },
    // };
    
    
    // public static readonly DifferenceColorCollection Dark = new()
    // {
    //     Added = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#163820")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#1F5A32")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#4ADE80")),
    //     },
    //     Removed = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#3A1A1D")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#6B2A2F")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#FF6B6B")),
    //     },
    //     Modified = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#3A2F12")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#6B5418")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#FACC15")),
    //     },
    //     Placeholder = new DifferenceColor
    //     {
    //         Background = new ImmutableSolidColorBrush(Color.Parse("#12324F")),
    //         HighLight = new ImmutableSolidColorBrush(Color.Parse("#1D4E78")),
    //         Mark = new ImmutableSolidColorBrush(Color.Parse("#60A5FA")),
    //     },
    // };
    
    public static readonly DifferenceColorCollection Dark = new()
    {
        Added = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#1B4D2A")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#2F7D46")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#5EEA7D")),
        },
        Removed = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#4A2024")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#8A343A")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#FF7B72")),
        },
        Modified = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#4A3A12")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#8A6A1E")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#FFD33D")),
        },
        Placeholder = new DifferenceColor
        {
            Background = new ImmutableSolidColorBrush(Color.Parse("#173F63")),
            HighLight = new ImmutableSolidColorBrush(Color.Parse("#256A9E")),
            Mark = new ImmutableSolidColorBrush(Color.Parse("#79C0FF")),
        },
    };


    public IBrush BackgroundColor(DifferenceLine.Kind kind)
    {
        var brush = kind switch
        {
            DifferenceLine.Kind.Modified => Modified.Background,
            DifferenceLine.Kind.Unchanged => new ImmutableSolidColorBrush(Colors.Transparent),
            DifferenceLine.Kind.Add => Placeholder.Background,
            DifferenceLine.Kind.Added => Added.Background,
            DifferenceLine.Kind.Remove => Removed.Background,
            DifferenceLine.Kind.Removed => Placeholder.Background,
            DifferenceLine.Kind.Visual => new ImmutableSolidColorBrush(Colors.Transparent),
            _ => throw new ArgumentOutOfRangeException(nameof(kind), kind, "Invalid difference kind")
        };

        return brush;
    }

}

