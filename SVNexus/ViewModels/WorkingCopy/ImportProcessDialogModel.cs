using System;
using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using SVNexus.Views;

namespace SVNexus.ViewModels.WorkingCopy;

public partial class ImportProcessDialogModel : ViewModelBase
{
    public override Type? ViewType { get; set; } = typeof(ImportProcessDialog);
    
    
    public partial class StepItemViewModel: ViewModelLite
    {
        [ObservableProperty]
        public partial string Content { get; set; } = string.Empty;
    }


    public ObservableCollection<StepItemViewModel> Steps { get; } = [
        new()
        {
            Content = "test1"
        },
        new()
        {
            Content = "test2"
        }
    ];
    
    
    
    

}