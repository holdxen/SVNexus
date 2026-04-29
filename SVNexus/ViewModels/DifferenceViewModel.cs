using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
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
using SVNexus.Utils;

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

        public bool IsDelete => PropertyNewValue is null && PropertyOldValue is not null;
        public bool IsModified => PropertyNewValue is not null && PropertyOldValue is not null;
        public bool IsAdded => PropertyNewValue is not null && PropertyOldValue is null;
    }

    [ObservableProperty]
    public partial ObservableCollection<PropertyItemViewModel> PropertyItemViewModels { get; set; } = [];
    
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

    public Encoding TextEncoding { get; set; } = new UTF8Encoding(encoderShouldEmitUTF8Identifier: false, throwOnInvalidBytes: true);


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

    public async Task CompareProperty(string target, Revision peg, Revision? oldRevision, Revision? newRevision)
    {
        PropertyItemViewModels.Clear();
        var context = SendMessage(new OnGetContext()).Response;
        Dictionary<string, string>? oldProperties = null;
        if (oldRevision is not null)
        {
            var propertyListOptions = new PropertyListOptions(target, peg, oldRevision, Depth.Empty, false);
            oldProperties = (await context.PropertyList(propertyListOptions)).Entries.FirstOrDefault()?.Properties;
        }


        Dictionary<string, string>? newProperties = null;
        if (newRevision is not null)
        {
            var propertyListOptions = new PropertyListOptions(target, peg, newRevision, Depth.Empty, false);
            newProperties = (await context.PropertyList(propertyListOptions)).Entries.FirstOrDefault()?.Properties;
        }

        // var added = new Dictionary<string, string>();

        // var modified = new Dictionary<string, string>();
        newProperties ??= [];
        oldProperties ??= [];
        

        foreach (var (key, value) in oldProperties)
        {
            if (newProperties.TryGetValue(key, out var v))
            {
                if (v != value)
                {
                    PropertyItemViewModels.Add(new PropertyItemViewModel()
                    {
                        PropertyName = key,
                        PropertyNewValue = v,
                        PropertyOldValue = value
                    });
                }

                newProperties.Remove(key);
            }
            else
            {
                PropertyItemViewModels.Add(new PropertyItemViewModel()
                {
                    PropertyName = key,
                    PropertyNewValue = null,
                    PropertyOldValue = value
                });
            }
        }

        foreach (var (key, value) in newProperties)
        {
            PropertyItemViewModels.Add(new PropertyItemViewModel()
            {
                PropertyName = key,
                PropertyNewValue = value
            });
        }

        
        PropertyModified = oldProperties != newProperties;
        
    }

    public async Task Compare(string target, Revision peg, Revision? oldRevision, Revision? newRevision)
    {
        try
        {
            LoadingOrErrorState = LoadingOrErrorState.MakeLoading();
            var context = SendMessage(new OnGetContext()).Response;


            string? oldText = null;
            byte[]? oldBytes = null;

            try
            {
                if (oldRevision is not null)
                {
                    var catOptions = new CatOptions(target, peg, oldRevision, true, false);
                    oldBytes = (await context.Cat(catOptions)).Content;
            
                    Logger.Info("Start decoding");
                    oldText = TextEncoding.GetString(oldBytes);
                    Logger.Info("Finish decoding");
                }
            }
            catch (DecoderFallbackException)
            {
                IsOldBinary = true;
            }
        
        

            string? newText = null;
            byte[]? newBytes = null;

            try
            {
                if (newRevision is not null)
                {
                    var catOptions = new CatOptions(target, peg, newRevision, true, false);
                    newBytes = (await context.Cat(catOptions)).Content;
            
                    newText = TextEncoding.GetString(newBytes);
                }
            }
            catch (DecoderFallbackException)
            {
                IsNewBinary = true;
            }

            await HandleContent(oldText ?? string.Empty, newText ?? string.Empty);

            await CompareProperty(target, peg, oldRevision, newRevision);
        }
        finally
        {
            LoadingOrErrorState = new LoadingOrErrorState.None();
        }
    }

    public async Task HandleContent(string oldContent, string newContent)
    {
        var (original, modified) = await Task.Run(() =>
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

            // foreach (var change in changes)
            // {
            //     Console.WriteLine("Change: {0}", change);
            //     if (change.Modified.Len > 0 && change.Original.Len > 0)
            //     {
            //         foreach (var line in original.Skip((int)change.Original.Pos).Take((int)change.Original.Len))
            //         {
            //             line.DifferenceKind = DifferenceLine.Kind.Modified;
            //         }   
            //         foreach (var line in modified.Skip((int)change.Modified.Pos).Take((int)change.Modified.Len))
            //         {
            //             line.DifferenceKind = DifferenceLine.Kind.Modified;
            //         }   
            //     }
            // }
            //
            // var add = changes.Where(c => c.Original.Len == 0 && c.Modified.Len > 0)
            //     .OrderByDescending(c => c.Original.Pos).ToList();
            //
            // foreach (var change in add)
            // {
            //     foreach (var line in modified.Skip((int)change.Modified.Pos).Take((int)change.Modified.Len))
            //     {
            //         line.DifferenceKind = DifferenceLine.Kind.Added;
            //     }
            // }
            //
            // var remove = changes.Where(c => c.Original.Len > 0 && c.Modified.Len == 0)
            //     .OrderByDescending(c => c.Modified.Pos).ToList();
            //
            // foreach (var change in remove)
            // {
            //     foreach (var line in original.Skip((int)change.Original.Pos).Take((int)change.Original.Len))
            //     {
            //         line.DifferenceKind = DifferenceLine.Kind.Remove;
            //     }
            // }
            //
            // foreach (var change in add)
            // {
            //     original.InsertRange((int)change.Original.Pos, Enumerable.Repeat(new DifferenceLine()
            //     {
            //         Content = "",
            //         DifferenceKind = DifferenceLine.Kind.Add
            //     }, (int)change.Modified.Len));
            // }
            //
            // foreach (var change in remove)
            // {
            //     modified.InsertRange((int)change.Modified.Pos, Enumerable.Repeat(new DifferenceLine()
            //     {
            //         Content = "",
            //         DifferenceKind = DifferenceLine.Kind.Removed
            //     }, (int)change.Original.Len));
            // }
            foreach (var change in changes)
            {
                switch (change.Original.Len)
                {
                    // added
                    case 0 when change.Modified.Len > 0:
                    {
                        // Console.WriteLine("Go change: {0}", change);
                        // Console.WriteLine("original: len={0}, index={1}", original.Count, original.ExcludeIndexToRealIndex((int)change.Original.Pos, [DifferenceLine.Kind.Add]));
                        // if (original.Count == 347)
                        // {
                        //     var i = 0;
                        //     foreach (var line in original)
                        //     {
                        //         Console.WriteLine("line: {0} {1} {2}", i, line.DifferenceKind, line.Content);
                        //         if (line.DifferenceKind == DifferenceLine.Kind.Add)
                        //         {
                        //             continue;
                        //         }   
                        //         i++;
                        //     }
                        // }
                        original.InsertRange(original.RealIndex((int)change.Original.Pos),
                            Enumerable.Repeat(new DifferenceLine()
                            {
                                Content = null,
                                DifferenceKind = DifferenceLine.Kind.Add
                            }, (int)change.Modified.Len));
            
            
                        foreach (var differenceLine in modified
                                     .Skip(modified.RealIndex((int)change.Modified.Pos))
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
                                     .Skip(original.RealIndex((int)change.Original.Pos)).Take((int)change.Original.Len))
                        {
                            differenceLine.DifferenceKind = DifferenceLine.Kind.Remove;
                        }
            
                        modified.InsertRange(modified.RealIndex((int)change.Modified.Pos),
                            Enumerable.Repeat(new DifferenceLine()
                            {
                                Content = null,
                                DifferenceKind = DifferenceLine.Kind.Removed
                            }, (int)change.Original.Len));
                        break;
                    }
                    case > 0 when change.Modified.Len > 0:
                    {
                        
                        {
                            var pos = (int)change.Original.Pos;
                            var len = (int)change.Original.Len;
                            var index = original.RealIndex(pos);
                            foreach (var differenceLine in original.Skip(index).Take(len))
                            {
                                differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
                            }
                            if (change.Original.Len < change.Modified.Len)
                            {
                                original.InsertRange(index+len, Enumerable.Repeat(new DifferenceLine()
                                {
                                    Content = null,
                                    DifferenceKind = DifferenceLine.Kind.Modified
                                }, (int)(change.Modified.Len - change.Original.Len)));
                            }
                        }
                        {
                            var pos = (int)change.Modified.Pos;
                            var len = (int)change.Modified.Len;
                            var index = modified.RealIndex(pos);
                            foreach (var differenceLine in modified.Skip(index).Take(len))
                            {
                                differenceLine.DifferenceKind = DifferenceLine.Kind.Modified;
                            }
                            if (change.Original.Len > change.Modified.Len)
                            {
                                modified.InsertRange(index+len, Enumerable.Repeat(new DifferenceLine()
                                {
                                    Content = null,
                                    DifferenceKind = DifferenceLine.Kind.Modified
                                }, (int)(change.Original.Len - change.Modified.Len)));
                            }
                        }
                        break;
                    }
                }
            }
            return new Tuple<List<DifferenceLine>, List<DifferenceLine>>(original, modified);
        });
        OldLines = original;
        NewLines = modified;
    }


    [RelayCommand]
    public async Task CompareWorkingCopyEntry(StatusEntry statusEntry)
    {
        
        TextModified = statusEntry.TextStatus is WorkingCopyStatus.Modified or WorkingCopyStatus.Conflicted;
        PropertyModified = statusEntry.PropertyStatus is WorkingCopyStatus.Modified or WorkingCopyStatus.Conflicted;
        
        if (statusEntry.NodeKind == NodeKind.Directory)
        {
            LoadingOrErrorState = new LoadingOrErrorState.None();
            OldLines = [];
            NewLines = [];
            if (statusEntry.NodeStatus != WorkingCopyStatus.Unversioned)
            {
                await CompareProperty(statusEntry.Path, 
                    new Revision.Base(), 
                    statusEntry.NodeStatus == WorkingCopyStatus.Added ? null : new Revision.Base(), 
                    statusEntry.NodeStatus == WorkingCopyStatus.Deleted ? null : new Revision.Working()
                );
            }

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
                    catOriginal = () => Task.FromResult(new CatResult([], []));
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
            
            await HandleContent(TextEncoding.GetString(resultOriginal.Content), TextEncoding.GetString(resultModified.Content));

            if (statusEntry.NodeStatus != WorkingCopyStatus.Unversioned)
            {
                await CompareProperty(statusEntry.Path, 
                    new Revision.Base(), 
                    statusEntry.NodeStatus == WorkingCopyStatus.Added ? null : new Revision.Base(), 
                    statusEntry.NodeStatus == WorkingCopyStatus.Deleted ? null : new Revision.Working()
                );
            }


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