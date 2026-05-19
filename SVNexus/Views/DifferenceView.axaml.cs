using System;
using System.Collections.Generic;
using System.Linq;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Data;
using Avalonia.Interactivity;
using Avalonia.VisualTree;
using AvaloniaEdit;
using AvaloniaEdit.Rendering;
using SVNexus.Components;
using SVNexus.Models;
using SVNexus.Utils;
using SVNexus.ViewModels;

namespace SVNexus.Views;


public sealed class DiffScrollSynchronizer
{
    private ScrollViewer? _leftSv;
    private ScrollViewer? _rightSv;
    private bool _syncing;

    public DiffScrollSynchronizer(TextEditor left, TextEditor right)
    {
        Hook(left, isLeft: true);
        Hook(right, isLeft: false);
    }

    private void Hook(TextEditor editor, bool isLeft)
    {
        // 如果模板已经应用过，直接拿；否则等 TemplateApplied
        var sv = editor.FindDescendantOfType<ScrollViewer>();
        if (sv != null)
        {
            Bind(sv, isLeft);
        }
        else
        {
            editor.TemplateApplied += (_, e) =>
            {
                var found = e.NameScope.Find<ScrollViewer>("PART_ScrollViewer")
                            ?? editor.FindDescendantOfType<ScrollViewer>();
                if (found != null) Bind(found, isLeft);
            };
        }
    }

    private void Bind(ScrollViewer sv, bool isLeft)
    {
        if (isLeft) _leftSv = sv; else _rightSv = sv;

        // ScrollChanged 在 Offset / Extent / Viewport 任意变化时触发
        sv.ScrollChanged += (_, _) =>
        {
            if (isLeft) SyncTo(_leftSv, _rightSv);
            else        SyncTo(_rightSv, _leftSv);
        };
    }

    private void SyncTo(ScrollViewer? source, ScrollViewer? target)
    {
        if (_syncing || source is null || target is null) return;

        _syncing = true;
        try
        {
            var srcMaxX = source.Extent.Width  - source.Viewport.Width;
            var srcMaxY = source.Extent.Height - source.Viewport.Height;
            var dstMaxX = target.Extent.Width  - target.Viewport.Width;
            var dstMaxY = target.Extent.Height - target.Viewport.Height;

            // 按比例换算，extent==viewport 时归零，避免除 0
            var px = srcMaxX > 0 ? source.Offset.X / srcMaxX : 0;
            var py = srcMaxY > 0 ? source.Offset.Y / srcMaxY : 0;

            var newX = dstMaxX > 0 ? dstMaxX * px : 0;
            var newY = dstMaxY > 0 ? dstMaxY * py : 0;

            // 只有真正变化时才赋值，进一步减少回环
            if (Math.Abs(target.Offset.X - newX) > 0.5 ||
                Math.Abs(target.Offset.Y - newY) > 0.5)
            {
                target.Offset = new Vector(newX, newY);
            }
        }
        finally
        {
            _syncing = false;
        }
    }
}


    public class CheckBoxColumn : Canvas
    {
        public static readonly StyledProperty<List<bool?>> LinesProperty = AvaloniaProperty.Register<CheckBoxColumn, List<bool?>>(
            nameof(Lines), [], defaultBindingMode: BindingMode.TwoWay);

        public List<bool?> Lines
        {
            get => GetValue(LinesProperty);
            set => SetValue(LinesProperty, value);
        }

        public static readonly StyledProperty<List<DifferenceLine>> TextLinesProperty = AvaloniaProperty.Register<CheckBoxColumn, List<DifferenceLine>>(
            nameof(TextLines), []);

        public List<DifferenceLine> TextLines
        {
            get => GetValue(TextLinesProperty);
            set => SetValue(TextLinesProperty, value);
        }

        public static readonly StyledProperty<bool> ChangeOnlyProperty = AvaloniaProperty.Register<CheckBoxColumn, bool>(
            nameof(ChangeOnly));

        public bool ChangeOnly
        {
            get => GetValue(ChangeOnlyProperty);
            set => SetValue(ChangeOnlyProperty, value);
        }

        public static readonly StyledProperty<bool> SelectLineProperty = AvaloniaProperty.Register<CheckBoxColumn, bool>(
            nameof(SelectLine));

        public bool SelectLine
        {
            get => GetValue(SelectLineProperty);
            set => SetValue(SelectLineProperty, value);
        }
        
        public AdvancedEditor? Editor { get; set; }

        private readonly Dictionary<CheckBox, int> _mapper = [];
        
        private int RealIndex(int index)
        {
            if (!ChangeOnly)
            {
                return index;
            }
            var count = 0;
            for (var i = 0; i < TextLines.Count; i++)
            {
                if (TextLines[i].DifferenceKind is DifferenceLine.Kind.Unchanged)
                {
                    continue;
                }
                if (index == count)
                {
                    return i;
                }
                count++;
            }
        
            // throw new IndexOutOfRangeException();
            return -1;
        }

        public void Link()
        {
            if (Editor is null)
            {
                return;
            }
            
            var view = Editor.TextArea.TextView;
            
            view.ScrollOffsetChanged += (_, _) =>
            {
                Rebuild();  
            };
            view.VisualLinesChanged += (_, _) =>
            {
                Rebuild();
            };
            // view.LayoutUpdated += (_, _) =>
            // {
            //     Rebuild();
            // };
        }

        protected override void OnPropertyChanged(AvaloniaPropertyChangedEventArgs change)
        {
            base.OnPropertyChanged(change);
            if (change.Property == LinesProperty || change.Property == TextLinesProperty || change.Property == ChangeOnlyProperty)
            {
                Rebuild();
            }
        }

        public void Rebuild()
        {
            Children.Clear();
            _mapper.Clear();


            var view = Editor?.TextArea.TextView;
            
            if (view is not { VisualLinesValid: true })
            {
                return;
            }

            if (TextLines.Count != Lines.Count)
            {
                Logger.Info($"Not equal number of lines: {TextLines.Count} {Lines.Count}");
                return;
            }

            if (Lines.Count == 0)
            {
                return;
            }
            
            Logger.Info($"Render {view.VisualLines.Count}");
            foreach (var line in view.VisualLines)
            {
                var lineNumber = line.FirstDocumentLine.LineNumber - 1;
                
                lineNumber = RealIndex(lineNumber);

                if (lineNumber < 0 || lineNumber >= Lines.Count)
                {
                    Logger.Error($"Line number {lineNumber} is out of range");
                    continue;
                }
                
                var state = Lines[lineNumber];

                if (state is null)
                {
                    continue;
                }
                
                var top = line.GetTextLineVisualYPosition(line.TextLines[0], VisualYPosition.TextTop);
                var bottom = line.GetTextLineVisualYPosition(line.TextLines[^1], VisualYPosition.TextBottom);

                if (bottom < view.VerticalOffset)
                {
                    continue;
                }
                
                var s = Math.Min(bottom - top, Bounds.Width);

                var x = (Bounds.Width - s) / 2;
                var y = top - view.VerticalOffset;
                
                Logger.Info($"Add Checkbox: x={x}, y={y}, s={s}");
                var checkBox = new CheckBox()
                {
                    IsChecked = state,
                    Width = s,
                    Height = s
                };
                
                
                checkBox.IsCheckedChanged += CheckBoxOnIsCheckedChanged;
                _mapper[checkBox] = lineNumber;
                
                Children.Add(checkBox);
                
                SetLeft(checkBox, x);
                SetTop(checkBox, y);
            }
        }

        private void CheckBoxOnIsCheckedChanged(object? sender, RoutedEventArgs e)
        {
            if (sender is not CheckBox checkBox)
            {
                return;
            }

            if (!_mapper.TryGetValue(checkBox, out var index))
            {
                return;
            }
            Lines[index] = checkBox.IsChecked;

            if (SelectLine) return;
            for (var i = index + 1; i < Lines.Count; i++)
            {
                if (Lines[i] is null)
                {
                    break;
                }
                Lines[i] = checkBox.IsChecked;
            }
            for (var i = index - 1; i >= 0; i--)
            {
                if (Lines[i] is null)
                {
                    break;
                }
                Lines[i] = checkBox.IsChecked;
            }
            Lines = new List<bool?>(Lines);
            Rebuild();
        }
    }

public partial class DifferenceView : UserControl
{
    public DifferenceView()
    {
        InitializeComponent();
        _ = new DiffScrollSynchronizer(OldTextEditor, NewTextEditor);
        CheckBoxColumn.Editor = OldTextEditor;
        CheckBoxColumn.Link();
    }


#if false
    
    private void NewTextViewOnScrollOffsetChanged(object? sender, EventArgs e)
    {
        // NewTextEditor.ScrollToHorizontalOffset(OldTextEditor.HorizontalOffset);
        // NewTextEditor.ScrollToVerticalOffset(OldTextEditor.VerticalOffset);
        
        var sv = OldTextEditor.GetVisualDescendants()
            .OfType<ScrollViewer>()
            .FirstOrDefault();

        OldTextEditor.TextArea.TextView.ScrollOffsetChanged -= OldTextViewOnScrollOffsetChanged;
        sv?.Offset = new Vector(NewTextEditor.HorizontalOffset, NewTextEditor.VerticalOffset);
        OldTextEditor.TextArea.TextView.ScrollOffsetChanged += OldTextViewOnScrollOffsetChanged;
    }

    private void OldTextViewOnScrollOffsetChanged(object? sender, EventArgs e)
    {
        // NewTextEditor.ScrollToHorizontalOffset(OldTextEditor.HorizontalOffset);
        // NewTextEditor.ScrollToVerticalOffset(OldTextEditor.VerticalOffset);
        
        var sv = NewTextEditor.GetVisualDescendants()
            .OfType<ScrollViewer>()
            .FirstOrDefault();
        
        NewTextEditor.TextArea.TextView.ScrollOffsetChanged -= NewTextViewOnScrollOffsetChanged;
        
        sv?.Offset = new Vector(OldTextEditor.HorizontalOffset, OldTextEditor.VerticalOffset);
        NewTextEditor.TextArea.TextView.ScrollOffsetChanged += NewTextViewOnScrollOffsetChanged;
    }
#endif
}