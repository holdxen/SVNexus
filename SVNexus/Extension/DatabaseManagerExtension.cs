using SVNexus.Generated;

namespace SVNexus.Extension;

public static class DatabaseManagerExtension
{
    private static readonly DatabaseManager DatabaseManager = new();
    extension(DatabaseManager)
    {
        public static DatabaseManager Default => DatabaseManager;
    }
}