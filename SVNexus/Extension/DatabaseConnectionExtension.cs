using System;
using System.Threading.Tasks;
using SVNexus.Generated;

namespace SVNexus.Extension;

public static class DatabaseConnectionExtension
{
    private static SeaDatabaseConnection? _databaseManager;

    public static async Task Create()
    {
        _databaseManager = await SeaDatabaseConnection.Create();
    }
    
    extension(SeaDatabaseConnection)
    {
        public static SeaDatabaseConnection Default => _databaseManager ?? throw new InvalidOperationException();
    }
}