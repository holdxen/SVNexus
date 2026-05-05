using System;
using Avalonia.Controls;
using Avalonia.Controls.Primitives;
using Avalonia.Input;
using Avalonia.VisualTree;
using SVNexus.Utils;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.ViewModels.WorkingCopy.History;

namespace SVNexus.Views.WorkingCopy.History;

public partial class HistoryView : UserControl
{
    public HistoryView()
    {
        InitializeComponent();
    }
    
    
    private ScrollBar? _verticalScrollbar;


    private void OnScrollChanged(object? sender, ScrollEventArgs e)
    {
        if (_verticalScrollbar is null || DataContext is not HistoryViewModel vm)
        {
            return;
        }
        if (_verticalScrollbar.Value / _verticalScrollbar.Maximum < 0.1)
        {
            vm.OnLoadTopMore();
        }
        if (_verticalScrollbar.Value / _verticalScrollbar.Maximum > 0.9)
        {
            vm.OnLoadBottomMore();
        }
    }

    private void OnWheelChanged(object? sender, PointerWheelEventArgs e)
    {
        if (_verticalScrollbar is null || DataContext is not HistoryViewModel vm)
        {
            return;
        }

        switch (e.Delta.Y)
        {
            case > 0:
            {
                if (_verticalScrollbar.Value / _verticalScrollbar.Maximum < 0.1)
                {
                    vm.OnLoadTopMore();
                }

                break;
            }
            case < 0:
            {
                if (_verticalScrollbar.Value / _verticalScrollbar.Maximum > 0.9)
                {
                    vm.OnLoadBottomMore();
                }

                break;
            }
        }
    }

    private void DataGrid_OnTemplateApplied(object? sender, TemplateAppliedEventArgs e)
    {
        if (sender is not DataGrid dataGrid) return;

        // dataGrid.PointerWheelChanged += (o, args) =>
        // {
        //     Logger.Info($"Wheel changed: {args.Delta}");
        // };
        //
        

        foreach (var control in dataGrid.GetVisualDescendants())
        {
            switch (control)
            {
                case DataGridRowsPresenter dataGridRowPresenter when control.Name == "PART_RowsPresenter":
                    dataGridRowPresenter.PointerWheelChanged += OnWheelChanged;
                    break;
                case ScrollBar scrollBar:
                {
                    if (control.Name == "PART_VerticalScrollbar")
                    {
                        _verticalScrollbar = scrollBar;
                        _verticalScrollbar.Scroll += OnScrollChanged;
                    }

                    break;
                }
            }
        }

    }

}