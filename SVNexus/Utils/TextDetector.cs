
using System;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using System.Text;

namespace SVNexus.Utils;

public static class TextDetector
{
    public static bool IsText(byte[] data, [MaybeNullWhen(false)] out Encoding encoding)
    {
        encoding = null;

        if (data.Length == 0)
        {
            encoding = Encoding.UTF8;
            return true;
        }

        // 1) BOM 检测
        if (HasBom(data, Encoding.UTF8))
        {
            encoding = Encoding.UTF8;
            return IsDecodableText(data, new UTF8Encoding(true, true), bomLength: 3);
        }

        if (HasBom(data, Encoding.Unicode)) // UTF-16 LE
        {
            encoding = Encoding.Unicode;
            return IsDecodableText(data, new UnicodeEncoding(false, true, true), bomLength: 2);
        }

        if (HasBom(data, Encoding.BigEndianUnicode)) // UTF-16 BE
        {
            encoding = Encoding.BigEndianUnicode;
            return IsDecodableText(data, new UnicodeEncoding(true, true, true), bomLength: 2);
        }

        switch (data)
        {
            case [0xFF, 0xFE, 0x00, 0x00, ..]:
                encoding = Encoding.UTF32;
                return IsDecodableText(data, new UTF32Encoding(false, true, true), bomLength: 4);
            case [0x00, 0x00, 0xFE, 0xFF, ..]:
                encoding = new UTF32Encoding(true, true, true);
                return IsDecodableText(data, new UTF32Encoding(true, true, true), bomLength: 4);
        }

        // 2) 粗略过滤明显的二进制：大量 NUL 字节
        // 纯 UTF-16 文本没有 BOM 时可能也会有 0x00，所以这里只做“明显异常”判断
        var nullCount = data.Count(b => b == 0);
        if (nullCount > data.Length / 4)
        {
            return false;
        }

        // 3) 严格按 UTF-8 尝试
        var utf8Strict = new UTF8Encoding(false, true);
        if (IsDecodableText(data, utf8Strict, bomLength: 0))
        {
            encoding = Encoding.UTF8;
            return true;
        }

        // 4) 可选：尝试 UTF-16（无 BOM）
        var utf16Le = new UnicodeEncoding(false, true, true);
        if (IsDecodableText(data, utf16Le, bomLength: 0))
        {
            encoding = Encoding.Unicode;
            return true;
        }

        var utf16Be = new UnicodeEncoding(true, true, true);
        if (!IsDecodableText(data, utf16Be, bomLength: 0)) return false;
        encoding = Encoding.BigEndianUnicode;
        return true;

    }

    private static bool HasBom(byte[] data, Encoding encoding)
    {
        var bom = encoding.GetPreamble();
        if (bom.Length == 0 || data.Length < bom.Length)
            return false;

        return !bom.Where((t, i) => data[i] != t).Any();
    }

    private static bool IsDecodableText(byte[] data, Encoding encoding, int bomLength)
    {
        try
        {
            var text = encoding.GetString(data, bomLength, data.Length - bomLength);

            if (string.IsNullOrEmpty(text))
                return true;

            var controlCount = text.Where(c => c != '\r' && c != '\n' && c != '\t').Count(char.IsControl);

            // 控制字符太多，通常不像正常文本
            return controlCount <= text.Length * 0.01;
        }
        catch
        {
            return false;
        }
    }
}
