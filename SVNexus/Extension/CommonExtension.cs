using System;

namespace SVNexus.Extension;

public static class CommonExtension
{
    extension<T>(T obj)
    {
        public T Apply(Action<T> action)
        {
            action(obj);
            return obj;
        }

        public TR Map<TR>(Func<T, TR> func)
        {
            return func(obj);
        }
    }
}