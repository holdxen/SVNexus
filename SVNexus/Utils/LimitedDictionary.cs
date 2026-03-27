using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using HarfBuzzSharp;

namespace SVNexus.Utils;

public class LimitedDictionary<TKey, TValue> where TKey : notnull
{
    public Dictionary<TKey, TValue> Dictionary { get; init; } = new();
    
    public int Limit { get; set; }


    public KeyValuePair<TKey, TValue>? Add(TKey key, TValue value)
    {
        if (Limit <= 0)
        {
            Dictionary.Clear();
            return null;
        }
        Dictionary[key] = value;

        if (Dictionary.Count < Limit) return null;
        
        var first = Dictionary.First();
        Dictionary.Remove(first.Key);
        return first;

    }

    public bool TryGetValue(TKey key, [MaybeNullWhen(false)] out TValue value)
    {
        return Dictionary.TryGetValue(key, out value);
    }

    public bool Remove(TKey key)
    {
        return Dictionary.Remove(key);
    }
    
    
    public TValue this[TKey key]
    {
        get => Dictionary[key];
        set => Dictionary[key] = value;
    }
}