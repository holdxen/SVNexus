using SVNexus.Generated;

namespace SVNexus.Extension;

public static class SvnErrnoConstantsExtension
{
    private static readonly SvnErrnoConstants Default = new();
    
    extension(SvnErrnoConstants)
    {
        public static SvnErrnoConstants Default => Default;
    }
}