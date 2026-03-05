using System;
using System.Runtime.InteropServices.Marshalling;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.VisualTree;
using SVNexus.ViewModels.WorkingCopy.Remote;

namespace SVNexus.Views.WorkingCopy.Remote;

public partial class RemoteView : UserControl
{
    public RemoteView()
    {
        InitializeComponent();
    }

    private void DataGrid_OnTemplateApplied(object? sender, TemplateAppliedEventArgs e)
    {
        if (sender is not DataGrid dataGrid) return;

        foreach (var control in dataGrid.GetVisualDescendants())
        {
            if (control is not ScrollBar scrollBar || control.Name != "PART_VerticalScrollbar") continue;
            scrollBar.ValueChanged += (o, args) =>
            {
                if (DataContext is RemoteViewModel vm)
                {
                    vm.OnDataGridVerticalScrollValueChanged(args.NewValue, scrollBar.Maximum);
                }
                Console.WriteLine("OnDataGridVerticalScroll: NewValue={0}, OldValue={1} Maximum={2}", args.NewValue, args.OldValue, scrollBar.Maximum);
            };
            break;
        }

    }

}