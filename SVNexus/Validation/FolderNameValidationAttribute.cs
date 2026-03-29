namespace SVNexus.Validation;

using System;
using System.ComponentModel.DataAnnotations;
using System.IO;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text.RegularExpressions;

/// <summary>
/// 验证字符串是否为合法的文件夹名称（跨平台）。
/// 综合考虑 Windows / Linux / macOS 的命名限制。
/// </summary>
[AttributeUsage(AttributeTargets.Property | AttributeTargets.Field | AttributeTargets.Parameter, AllowMultiple = false)]
public sealed class FolderNameValidationAttribute() : ValidationAttribute("'{0}' is not a valid folder name.")
{
    // Windows 文件系统禁止的字符
    private static readonly char[] WindowsInvalidChars =
        ['\\', '/', ':', '*', '?', '"', '<', '>', '|'];

    // Unix 系统禁止的字符
    private static readonly char[] UnixInvalidChars = ['/', '\0'];

    // Windows 保留设备名（不区分大小写）
    private static readonly string[] WindowsReservedNames =
    [
        "CON", "PRN", "AUX", "NUL",
        "COM0","COM1","COM2","COM3","COM4","COM5","COM6","COM7","COM8","COM9",
        "LPT0","LPT1","LPT2","LPT3","LPT4","LPT5","LPT6","LPT7","LPT8","LPT9"
    ];

    /// <summary>
    /// 是否强制执行所有平台的规则（默认 true）。
    /// 设为 false 时仅校验当前运行平台的规则。
    /// </summary>
    public bool EnforceAllPlatforms { get; set; } = true;

    /// <summary>
    /// 允许的最大文件夹名长度。Windows 默认 255，大多数 Linux/macOS 文件系统也是 255。
    /// </summary>
    public int MaxLength { get; set; } = 255;

    protected override ValidationResult? IsValid(object? value, ValidationContext validationContext)
    {
        // null 值交给 [Required] 处理
        if (value is null)
            return ValidationResult.Success;

        if (value is not string folderName)
            return new ValidationResult("值必须为字符串类型。");

        var error = Validate(folderName);
        if (error is null) return ValidationResult.Success;
        var memberName = validationContext.MemberName;
        return new ValidationResult(
            string.Format(ErrorMessageString, memberName ?? "值") + " " + error,
            memberName is not null ? new[] { memberName } : Array.Empty<string>());

    }

    /// <summary>
    /// 核心校验逻辑，返回 null 表示合法，否则返回错误描述。
    /// </summary>
    public string? Validate(string folderName)
    {
        // 1. 空白检查
        if (string.IsNullOrWhiteSpace(folderName))
            return "文件夹名称不能为空或纯空白字符。";

        // 2. 长度检查
        if (folderName.Length > MaxLength)
            return $"文件夹名称长度不能超过 {MaxLength} 个字符。";

        // 3. 不能包含控制字符（ASCII 0-31）
        if (folderName.Any(c => c < 32))
            return "文件夹名称不能包含控制字符。";

        bool checkWindows = EnforceAllPlatforms || RuntimeInformation.IsOSPlatform(OSPlatform.Windows);
        bool checkUnix    = EnforceAllPlatforms || !RuntimeInformation.IsOSPlatform(OSPlatform.Windows);

        // 4. 非法字符检查
        if (checkWindows)
        {
            var found = folderName.IndexOfAny(WindowsInvalidChars);
            if (found >= 0)
                return $"文件夹名称不能包含字符 '{folderName[found]}'。";
        }

        if (checkUnix)
        {
            var found = folderName.IndexOfAny(UnixInvalidChars);
            if (found >= 0)
                return $"文件夹名称不能包含字符 '\\x{(int)folderName[found]:X2}'。";
        }

        // 5. Windows 特有规则
        if (!checkWindows) return folderName is "." or ".." ? "文件夹名称不能是 '.' 或 '..'。" : null; // 合法
        // 不能以点或空格结尾
        if (folderName.EndsWith('.') || folderName.EndsWith(' '))
            return "文件夹名称不能以点号 (.) 或空格结尾。";

        // 不能使用保留设备名（如 CON、NUL、COM1 等，含带扩展名的情况）
        var nameWithoutExt = folderName.Contains('.')
            ? folderName[..folderName.IndexOf('.')]
            : folderName;

        if (WindowsReservedNames.Any(r =>
                string.Equals(r, nameWithoutExt, StringComparison.OrdinalIgnoreCase)))
            return $"'{nameWithoutExt}' 是 Windows 保留名称，不能用作文件夹名。";

        // 6. 不能是 . 或 ..（所有平台）
        return folderName is "." or ".." ? "文件夹名称不能是 '.' 或 '..'。" : null; // 合法
    }
}
