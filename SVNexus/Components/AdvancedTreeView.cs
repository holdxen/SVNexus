using System;
using Avalonia.Controls;

namespace SVNexus.Components;

public class AdvancedTreeView: TreeView
{
    protected override Type StyleKeyOverride { get; } = typeof(TreeView);
    
    protected override Control CreateContainerForItemOverride(object? item, int index, object? recycleKey)
    {
        return new AdvancedTreeViewItem();
    }
    
    protected override bool NeedsContainerOverride(
        object? item,
        int index,
        out object? recycleKey)
    {
        return NeedsContainer<AdvancedTreeViewItem>(item, out recycleKey);
    }
}