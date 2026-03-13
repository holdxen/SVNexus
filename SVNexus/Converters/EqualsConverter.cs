namespace SVNexus.Converters;

using System;
using System.Globalization;
using Avalonia.Data.Converters;

public class EqualsConverter : IValueConverter
{
    public static EqualsConverter Default { get; } = new();
    
    public object Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        // parameter 写在 XAML 里，如 ConverterParameter=SomeValue
        if (value is null && parameter is null) return true;
        if (value is null || parameter is null) return false;

        // 需要时可在这里做类型转换（如枚举、数字、字符串）
        return value.Equals(parameter);
    }

    public object ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
        => throw new NotSupportedException();
}