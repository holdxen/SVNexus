using System;
using System.Text.Json;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Utils;

namespace SVNexus.Engine;

public class UpdateOperationDelegate: UpdateOperation
{
    public required Func<AnyValue, AnyValue> UpdateFunc { get; init; }
    
    public AnyValue Update(AnyValue v)
    {
        return UpdateFunc(v);
    }
}


public class UpdateOperationValue(AnyValue value): UpdateOperation
{

    public AnyValue Update(AnyValue v)
    {
        return value;
    }
}