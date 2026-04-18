using System;
using System.Collections.Generic;
using System.IO;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using SVNexus.Components;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Models;

namespace SVNexus.ViewModels;

public partial class DifferenceViewModel(ViewModelBase parent): ViewModelBase(parent)
{
    
    public enum Algorithm
    {
        Subversion,
    }

    public class PropertyItemViewModel
    {
        public required string PropertyName { get; set; }
        public string? PropertyOldValue { get; set; }
        public string? PropertyNewValue { get; set; }
        
        public bool IsDelete { get; set; }
        public bool Modified { get; set; }
        public bool IsAdded { get; set; }
        public bool IsConflicted { get; set; }
    }
    
    [ObservableProperty]
    public partial LoadingOrErrorState LoadingOrErrorState { get; set; } = LoadingOrErrorState.MakeNone();
    
    // public bool IsTextView { get; set; }
    
    public string Path { get; set; } = string.Empty;
    
    public bool InOne { get; set; }
    
    public bool TextModified { get; set; } = false;
    
    public bool PropertyModified { get; set; } = false;
    
    public bool IsOldBinary { get; set; }
    
    public bool IsNewBinary { get; set; }

    [ObservableProperty]
    public partial List<DifferenceLine> OldLines { get; set; } = [];
    
    [ObservableProperty]
    public partial List<DifferenceLine> NewLines { get; set; } = [];

    private const int TextViewIndex = 0;
    private const int PropertyViewIndex = 1;
    
    public bool IsTextView => SelectedViewIndex == TextViewIndex;
    
    public bool IsPropertyView => SelectedViewIndex == PropertyViewIndex;

    [ObservableProperty]
    [NotifyPropertyChangedFor(nameof(IsTextView))]
    [NotifyPropertyChangedFor(nameof(IsPropertyView))]
    public partial int SelectedViewIndex { get; set; } = TextViewIndex;


    [RelayCommand]
    private void SwitchToPropertyView()
    {
        SelectedViewIndex = PropertyViewIndex;
    }

    [RelayCommand]
    private void SwitchToTextView()
    {
        SelectedViewIndex = TextViewIndex;
    }

    public void HandleContent(string oldContent, string newContent)
    {
        var modified = newContent.Split("\n").Select(e =>
            new DifferenceLine()
            {
                Content = e,
                DifferenceKind = DifferenceLine.Kind.Unchanged
            }).ToList();
        if (modified.Count > 0 && string.IsNullOrEmpty(modified.Last().Content))
        {
            modified.RemoveAt(modified.Count - 1);
        }

        var original = oldContent.Split("\n")
            .Select(e => new DifferenceLine()
            {
                Content = e,
                DifferenceKind = DifferenceLine.Kind.Unchanged
            }).ToList();
                
        if (original.Count > 0 && string.IsNullOrEmpty(original.Last().Content))
        {
            original.RemoveAt(original.Count - 1);
        }


        var diffOptions =
            new DifferenceOptions(Original: Encoding.UTF8.GetBytes(oldContent), Modified: Encoding.UTF8.GetBytes(newContent),
                Options: new DifferenceFileOptions(DiffFileIgnoreSpace.None, false, false, 0));

        var changes = diffOptions.Exec().Modified;

        foreach (var change in changes)
        {
            switch (change.Original.Len)
            {
                // Console.WriteLine("Go change: {0}", change);
                // added
                case 0 when change.Modified.Len > 0:
                {
                    original.InsertRange(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add]),
                        Enumerable.Repeat(new DifferenceLine()
                        {
                            Content = "",
                            DifferenceKind = DifferenceLine.Kind.Add
                        }, (int)change.Modified.Len));


                    foreach (var differenceLine in modified
                                 .Skip(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]))
                                 .Take((int)change.Modified.Len))
                    {
                        differenceLine.DifferenceKind = DifferenceLine.Kind.Added;
                    }

                    break;
                }
                // remove
                case > 0 when change.Modified.Len == 0:
                {
                    foreach (var differenceLine in original
                                 .Skip(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add])).Take((int)change.Original.Len))
                    {
                        differenceLine.DifferenceKind = DifferenceLine.Kind.Remove;
                    }

                    modified.InsertRange(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]),
                        Enumerable.Repeat(new DifferenceLine()
                        {
                            Content = "",
                            DifferenceKind = DifferenceLine.Kind.Removed
                        }, (int)change.Original.Len));
                    break;
                }
                case > 0 when change.Modified.Len > 0:
                {
                    {
                        var pos = (int)change.Original.Pos;
                        var len = (int)change.Original.Len;
                        foreach (var differenceLine in original
                                     .Skip(original.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Add]))
                                     .Take(len))
                        {
                            differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
                        }
                    }
                    {
                        var pos = (int)change.Modified.Pos;
                        var len = (int)change.Modified.Len;
                        foreach (var differenceLine in modified
                                     .Skip(modified.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Removed]))
                                     .Take(len))
                        {
                            differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
                        }
                    }
                    break;
                }
            }
        }
        OldLines = original;
        NewLines = modified;
    }

    [RelayCommand]
    public async Task CompareWorkingCopyEntry(StatusEntry statusEntry)
    {
        
        TextModified = statusEntry.TextStatus is WorkingCopyStatus.Modified or WorkingCopyStatus.Conflicted;
        PropertyModified = statusEntry.PropStatus is WorkingCopyStatus.Modified or WorkingCopyStatus.Conflicted;
        
        if (statusEntry.NodeKind == NodeKind.Directory)
        {
            LoadingOrErrorState = new LoadingOrErrorState.None();
            OldLines = [];
            NewLines = [];
            return;
        }
        
        LoadingOrErrorState = LoadingOrErrorState.MakeLoading();
        
        try
        {
            var context = SendMessage(new OnGetContext()).Response;

            Func<Task<CatResult>> catModified;
            Func<Task<CatResult>> catOriginal;

            switch (statusEntry.NodeStatus)
            {
                case WorkingCopyStatus.Missing or WorkingCopyStatus.Deleted:
                    catModified = () => Task.FromResult(new CatResult([], []));
                    catOriginal = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Base(),
                            Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
                        return context.Cat(catOptions);
                    };
                    break;
                case WorkingCopyStatus.Unversioned or WorkingCopyStatus.Added:
                    catModified = async () =>
                    {
                        var content = await File.ReadAllBytesAsync(statusEntry.Path);
                        return new  CatResult(content, []);
                    };
                    catOriginal = () => Task.FromResult(new  CatResult([], []));
                    break;
                default:
                    catModified = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                            Revision: new Revision.Working(), ExpandKeywords: true, GetProperties: false);
                        return context.Cat(catOptions);
                    };
                    catOriginal = () =>
                    {
                        var catOptions = new CatOptions(Path: statusEntry.Path, PegRevision: new Revision.Unspecified(),
                            Revision: new Revision.Base(), ExpandKeywords: true, GetProperties: false);
                        return context.Cat(catOptions);
                    };
                    break;
            }
                
            var resultModified = await catModified(); 
            var resultOriginal = await catOriginal();
            
            HandleContent(Encoding.UTF8.GetString(resultOriginal.Content), Encoding.UTF8.GetString(resultModified.Content));
                
            // var modified = Encoding.UTF8.GetString(resultModified.Content).Split("\n").Select(e =>
            //     new DifferenceLine()
            //     {
            //         Content = e,
            //         DifferenceKind = DifferenceLine.Kind.Unchanged
            //     }).ToList();
            // if (modified.Count > 0 && string.IsNullOrEmpty(modified.Last().Content))
            // {
            //     modified.RemoveAt(modified.Count - 1);
            // }
            //
            // var original = Encoding.UTF8.GetString(resultOriginal.Content).Split("\n")
            //     .Select(e => new DifferenceLine()
            //     {
            //         Content = e,
            //         DifferenceKind = DifferenceLine.Kind.Unchanged
            //     }).ToList();
            //     
            // if (original.Count > 0 && string.IsNullOrEmpty(original.Last().Content))
            // {
            //     original.RemoveAt(original.Count - 1);
            // }
            //
            //
            // var diffOptions =
            //     new DifferenceOptions(Original: resultOriginal.Content, Modified: resultModified.Content,
            //         Options: new DifferenceFileOptions(DiffFileIgnoreSpace.None, false, false, 0));
            //
            // var changes = diffOptions.Exec().Modified;
            //
            // // changes = [
            // //     new TextChange(Original: null, Modified: new TextPosition(3, 1)),
            // //     new TextChange(Original: null, Modified: new TextPosition(5, 1))
            // // ];
            //
            // foreach (var change in changes)
            // {
            //     // Console.WriteLine("Go change: {0}", change);
            //     if (change.Original.Len == 0 && change.Modified.Len > 0) // added
            //     {
            //         original.InsertRange(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add]),
            //             Enumerable.Repeat(new DifferenceLine()
            //             {
            //                 Content = "",
            //                 DifferenceKind = DifferenceLine.Kind.Add
            //             }, (int)change.Modified.Len));
            //
            //
            //         foreach (var differenceLine in modified
            //                      .Skip(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]))
            //                      .Take((int)change.Modified.Len))
            //         {
            //             differenceLine.DifferenceKind = DifferenceLine.Kind.Added;
            //         }
            //     }
            //     else if (change.Original.Len > 0 && change.Modified.Len == 0) // remove
            //     {
            //         foreach (var differenceLine in original
            //                      .Skip(original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add])).Take((int)change.Original.Len))
            //         {
            //             differenceLine.DifferenceKind = DifferenceLine.Kind.Remove;
            //         }
            //
            //         modified.InsertRange(modified.ExcludeIndexToRealIndex((int)change.Modified.Pos, [DifferenceLine.Kind.Removed]),
            //             Enumerable.Repeat(new DifferenceLine()
            //             {
            //                 Content = "",
            //                 DifferenceKind = DifferenceLine.Kind.Removed
            //             }, (int)change.Original.Len));
            //
            //     }
            //     else if (change.Original.Len > 0 && change.Modified.Len > 0)
            //     {
            //         {
            //             var pos = (int)change.Original.Pos;
            //             var len = (int)change.Original.Len;
            //             foreach (var differenceLine in original
            //                          .Skip(original.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Add]))
            //                          .Take(len))
            //             {
            //                 differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
            //             }
            //         }
            //         {
            //             var pos = (int)change.Modified.Pos;
            //             var len = (int)change.Modified.Len;
            //             foreach (var differenceLine in modified
            //                          .Skip(modified.ExcludeIndexToRealIndex(pos, [DifferenceLine.Kind.Removed]))
            //                          .Take(len))
            //             {
            //                 differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
            //             }
            //         }
            //     }
            // }
            // OldLines = original;
            // NewLines = modified;
            // var info = new Difference()
            // {
            //     Original = original,
            //     Modified = modified,
            // };

            LoadingOrErrorState = new LoadingOrErrorState.None();

        }
        catch (System.Exception e)
        {
            Console.WriteLine(e);
            LoadingOrErrorState = new LoadingOrErrorState.Error()
            {
                ErrorMessage = $"Failed to load content: {e.HumanReadableMessage}",
                RetryCommand = CompareWorkingCopyEntryCommand,
                RetryCommandParameter = statusEntry
            };
        }
    }
}