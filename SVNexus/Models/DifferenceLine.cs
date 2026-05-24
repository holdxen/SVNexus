using System;
using System.Collections.Generic;
using System.Text;

namespace SVNexus.Models;

public record DifferenceLine
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
        Crlf,
        Cr,
        None
    }
    
    public Kind DifferenceKind { get; set; }

    public string? Content { get; set; }
    
    public LineEnding Ending { get; set; } = LineEnding.None;
    
    public string? VisualText { get; set; }
    
    public string Text => Content ?? VisualText ?? string.Empty;

    public string EndingText()
    {
        return LineEndingText(Ending);
    }

    public static string LineEndingText(LineEnding ending)
    {
        return ending switch
        {
            LineEnding.Lf => "\n",
            LineEnding.Crlf => "\r\n",
            LineEnding.Cr => "\r",
            LineEnding.None => string.Empty,
            _ => throw new ArgumentOutOfRangeException(nameof(ending), ending, null)
        };
    }

    public static string ToText(IEnumerable<DifferenceLine> lines)
    {
        var builder = new StringBuilder();

        foreach (var line in lines)
        {
            builder.Append(line.Text);
            builder.Append(LineEndingText(line.Ending));
        }

        return builder.ToString();
    }

    public static IEnumerable<Tuple<string, LineEnding>> SplitLines(string text)
    {
        var start = 0;
        var i = 0;

        while (i < text.Length)
        {
            switch (text[i])
            {
                // CRLF
                case '\r' when i + 1 < text.Length && text[i + 1] == '\n':
                    yield return new Tuple<string, LineEnding>(
                        text.Substring(start, i - start),
                        LineEnding.Crlf
                    );

                    i += 2;
                    start = i;
                    break;
                case '\r':
                    // 单独 CR
                    yield return new  Tuple<string, LineEnding>(
                        text.Substring(start, i - start),
                        LineEnding.Cr
                    );

                    i++;
                    start = i;
                    break;
                case '\n':
                    // LF
                    yield return new  Tuple<string, LineEnding>(
                        text.Substring(start, i - start),
                        LineEnding.Lf
                    );

                    i++;
                    start = i;
                    break;
                default:
                    i++;
                    break;
            }
        }

        // 最后一行没有换行符
        if (start < text.Length)
        {
            yield return new  Tuple<string, LineEnding>(
                text[start..],
                LineEnding.None
            );
        }
        

    }
    
}