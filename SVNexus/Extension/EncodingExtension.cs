using System;
using System.Text;

namespace SVNexus.Extension;

public static class EncodingExtension
{
    extension(Encoding encoding)
    {
        public string? TryGetString(byte[] bytes)
        {
            try
            {
                return encoding.GetString(bytes);
            }
            catch (Exception)
            {
                return null;
            }
        }
    }
}