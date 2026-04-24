using System;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using OperationDepth = SVNexus.Generated.Depth;
namespace SVNexus.ViewModels;


public partial class RevertDialogModel(ViewModelBase parent): DialogModelBase(parent)
{
    public enum ValidDepth
    {
        Empty = OperationDepth.Empty,
        Files = OperationDepth.Files,
        Immediates = OperationDepth.Immediates,
        Infinity = OperationDepth.Infinity,
    }
    
    public static Type DepthType { get; } = typeof(ValidDepth);
    
    [ObservableProperty]
    public partial ValidDepth Depth { get; set; }
    
    protected override async Task OnConfirm()
    {
    }
}