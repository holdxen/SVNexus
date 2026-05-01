using System;
using System.Collections;
using System.Collections.Generic;
using System.Globalization;
using Avalonia.Data.Converters;
using SVNexus.Utils;

namespace SVNexus.Converters;

public class ListConverter: IValueConverter
{
    public static ListConverter IsEmptyOrNull { get; } = new()
    {
        ConvertFunc = list => list is null || list.Count == 0
    };
    
    public static ListConverter IsNotEmptyOrNull { get; } = new()
    {
        ConvertFunc = list => list is not null && list.Count != 0
    };
    
    public required Func<ICollection?, bool> ConvertFunc { get; init; }
    
    
    public object? Convert(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        Logger.Info($"value is {value is ICollection}");
        Console.WriteLine($"value is {value is ICollection}");
        return ConvertFunc.Invoke(value as ICollection);
    }

    public object? ConvertBack(object? value, Type targetType, object? parameter, CultureInfo culture)
    {
        throw new NotSupportedException();
    }
}
