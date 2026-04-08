namespace SVNexus.Utils;

using System;
using System.Diagnostics;
using System.Runtime.CompilerServices;

/// <summary>
/// Represents a void-like value for generic type parameters.
/// There is exactly one valid value: <see cref="Value"/>.
/// </summary>
[DebuggerDisplay("{ToString(),nq}")]
public readonly struct Unit : IEquatable<Unit>, IComparable<Unit>, IComparable
{
    /// <summary>
    /// The single value of <see cref="Unit"/>.
    /// </summary>
    public static readonly Unit Value = default;

    /// <summary>
    /// Always returns true because all <see cref="Unit"/> values are equal.
    /// </summary>
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public bool Equals(Unit other) => true;

    /// <summary>
    /// Returns true if <paramref name="obj"/> is a <see cref="Unit"/>.
    /// </summary>
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public override bool Equals(object? obj) => obj is Unit;

    /// <summary>
    /// Always returns 0 because all <see cref="Unit"/> values are equal.
    /// </summary>
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public override int GetHashCode() => 0;

    /// <summary>
    /// Always returns 0 because all <see cref="Unit"/> values are equal.
    /// </summary>
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public int CompareTo(Unit other) => 0;

    /// <summary>
    /// Compares this instance to another object.
    /// </summary>
    /// <exception cref="ArgumentException">
    /// Thrown when <paramref name="obj"/> is not null and not a <see cref="Unit"/>.
    /// </exception>
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public int CompareTo(object? obj)
    {
        return obj switch
        {
            null => 1,
            Unit => 0,
            _ => throw new ArgumentException($"Object must be of type {nameof(Unit)}.", nameof(obj))
        };
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public override string ToString() => "()";

    /// <summary>
    /// Supports deconstruction syntax.
    /// </summary>
    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public void Deconstruct()
    {
    }

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static bool operator ==(Unit left, Unit right) => true;

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static bool operator !=(Unit left, Unit right) => false;

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static bool operator <(Unit left, Unit right) => false;

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static bool operator >(Unit left, Unit right) => false;

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static bool operator <=(Unit left, Unit right) => true;

    [MethodImpl(MethodImplOptions.AggressiveInlining)]
    public static bool operator >=(Unit left, Unit right) => true;
}
