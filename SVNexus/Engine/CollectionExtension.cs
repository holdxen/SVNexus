using System;
using System.Collections.Generic;

namespace SVNexus.Engine;

public static class CollectionExtensions
{
    public static int FindIndex<T>(this IList<T> source, Func<T, bool> predicate)
    {
        for (var i = 0; i < source.Count; i++)
        {
            if (predicate(source[i]))
                return i;
        }
        return -1;  // 没找到返回 -1（和 List<T> 一致）
    }
}
