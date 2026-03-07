using System;

namespace SVNexus.Extension;

using Avalonia.Media;
using System.Globalization;

public static class ColorExtension
{
    extension(Color color)
    {
        public string ToCssHex()
        {
            return $"#{color.R:X2}{color.G:X2}{color.B:X2}{color.A:X2}";
        }

        public string ToCssRgba()
        {
            var alpha = color.A / 255.0;
            return FormattableString.Invariant(
                $"rgba({color.R}, {color.G}, {color.B}, {alpha:0.###})");
        }
    }

    // 输出 CSS rgba()：rgba(r, g, b, a)
}