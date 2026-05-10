using System;
using System.Threading;

namespace SVNexus.Utils;

public class LockedValue<T>(T v)
{
    private T _value = v;

    private readonly Lock _lock = new();
    
    public T Value
    {
        get
        {
            lock (_lock)
            {
                return _value;
            }
        }
        set
        {
            lock (_lock)
            {
                _value = value;
            }
        }
    }

    public T Take(Func<T, T> func)
    {
        lock (_lock)
        {
            var v = _value;
            _value = func(_value);
            return v;
        }
    }

}