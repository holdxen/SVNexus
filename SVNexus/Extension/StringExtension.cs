using System;
using System.IO;

namespace SVNexus.Extension;

public static class StringExtension
{
    extension(string self)
    {
        public string TrimStartString(string s, StringComparison comparisonType = StringComparison.Ordinal)
        {
            return self.StartsWith(s) ? self[s.Length..] : self;
        }
        
        public string GetFileName()
        {
            return Path.GetFileName(self.TrimEndPathSeparatorChar());
        }

        public string? GetDirectoryName()
        {
            return Path.GetDirectoryName(self.TrimEndPathSeparatorChar());
        }

        public string TrimEndPathSeparatorChar()
        {
            return self.TrimEnd(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        }
        
        public string TrimStartPathSeparatorChar()
        {
            return self.TrimStart(Path.DirectorySeparatorChar, Path.AltDirectorySeparatorChar);
        }
        
    }
}