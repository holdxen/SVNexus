using System;
using System.Collections.Generic;
using System.Collections.ObjectModel;
using System.Diagnostics;
using System.Globalization;
using System.Linq;
using System.Threading.Tasks;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Platform.Storage;
using Avalonia.Threading;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using CommunityToolkit.Mvvm.Messaging;
using SVNexus.Engine;
using SVNexus.Extension;
using SVNexus.Generated;
using SVNexus.Messages;
using SVNexus.Utils;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.Views;
using Ursa.Controls;

namespace SVNexus.ViewModels;

public partial class WelcomeViewModel(ViewModelBase parent): ViewModelBase(parent), 
    IRecipient<WelcomeViewModel.OnRemoveHistory>,
    IRecipient<WelcomeViewModel.OnHistoryStateChanged>
{

    public const int AllIndex = 0;
    public const int StarIndex = 1;
    public const int ModifiedIndex = 2;
    public const int ConflictedIndex = 3;
    public const int InvalidIndex = 4;

    public class OnRemoveHistory
    {
        public required HistoryItemViewModel Item { get; set; }
    }

    public class OnHistoryStateChanged;
    
    public abstract partial class HistoryItemViewModel(ViewModelBase parent): ViewModelMore(parent)
    {
        [ObservableProperty] 
        public partial bool IsVisible { get; set; } = true;
        
        public abstract bool IsStar { get; }
        
        public abstract string HistoryUuid { get; }
        
        public abstract string Icon { get; }
        
        [ObservableProperty]
        public partial bool IsEditing { get; set; }

        public abstract WorkspaceHistory RebuildWithIsStar(bool value);
        
        public abstract WorkspaceHistory RebuildWithIsRemark(string? value);
        
                
        [ObservableProperty]
        public partial string? Remark { get; set; }
        
        [RelayCommand]
        protected async Task ToggleIsStar()
        {
            var history = RebuildWithIsStar(!IsStar);
            
            await EngineBackend.Instance.DatabaseQueue.RunAndWait(async _ =>
            {
                await DatabaseManager.Default.SetWorkspaceHistory(history);
            });
            SendMessage(new OnHistoryStateChanged());
        }
        
        [RelayCommand]
        protected async Task Delete()
        {
            await EngineBackend.Instance.DatabaseQueue.RunAndWait(async _ =>
            {
                await DatabaseManager.Default.DeleteWorkspaceHistory(HistoryUuid);
            });
            SendMessage(new OnRemoveHistory()
            {
                Item = this
            });
        }
        
        [RelayCommand]
        private async Task EditRemark()
        {
            IsEditing = !IsEditing;
            if (IsEditing)
            {
                return;
            }

            var history = RebuildWithIsRemark(Remark);

            await EngineBackend.Instance.DatabaseQueue.RunAndWait(async _ =>
            {
                await DatabaseManager.Default.SetWorkspaceHistory(history);
            });
        }
        
    }

    public partial class RepositoryItemViewModel(ViewModelBase parent) : HistoryItemViewModel(parent)
    {
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsStar))]
        public required partial WorkspaceHistory.Repository History { get; set; }


        public string Name => "Repository";
        
        public string? LastUsedTime => History.LastUsedTime?.Map(seconds =>
        {
            var offset = DateTimeOffset.FromUnixTimeSeconds(seconds);
            return offset.ToLocalTime().DateTime.ToString(CultureInfo.InvariantCulture);
        });
        
        public override bool IsStar => History.Star;
        
        public override string HistoryUuid => History.Uuid;

        public string Url => History.RootUrl;

        public string RepositoryUuid => History.Uuid;
        
        
        public override string Icon => Application.Current?.FindResource("Icons.Repository") as string ?? string.Empty;
        public override WorkspaceHistory RebuildWithIsStar(bool value)
        {
            return History = History with { Star = value };
        }

        public override WorkspaceHistory RebuildWithIsRemark(string? value)
        {
            return History = History with { Remark = value };
        }

        public List<MenuItemViewModel> MenuItems =>
        [
            new()
            {
                Header = "Open",
                Command = OpenCommand
            },
            new()
            {
                Header = "Delete",
                Command = DeleteCommand
            },
        ];
        
        [RelayCommand]
        private void Open()
        {
        }
    }

    public partial class WorkingCopyItemViewModel(ViewModelBase parent) : HistoryItemViewModel(parent)
    {
        public override string Icon => Application.Current?.FindResource("Icons.WorkingCopy") as string ?? string.Empty;
        
        [ObservableProperty]
        [NotifyPropertyChangedFor(nameof(IsStar))]
        [NotifyPropertyChangedFor(nameof(LastUsedTime))]
        public required partial WorkspaceHistory.WorkingCopy History { get; set; }

        public string Name => History.WorkingCopyRoot.GetFileName();

        public string? LastUsedTime => History.LastUsedTime?.Map(seconds =>
        {
            var offset = DateTimeOffset.FromUnixTimeSeconds(seconds);
            return offset.ToLocalTime().DateTime.ToString(CultureInfo.InvariantCulture);
        });
        
        public string Path => History.WorkingCopyPath;
        
        public string? RepositoryRootUrl => History.RepositoryRootUrl;

        public override bool IsStar => History.Star;
        
        public override string HistoryUuid => History.Uuid;

        public List<MenuItemViewModel> MenuItems =>
        [
            new()
            {
                Header = "Open",
                Command = OpenCommand
            },
            new()
            {
                Header = "Delete",
                Command = DeleteCommand
            },
        ];
        
        // public string? Remark => History.Remark;

        public override WorkspaceHistory RebuildWithIsStar(bool value)
        {
            return History = History with { Star = value };
        }

        public override WorkspaceHistory RebuildWithIsRemark(string? value)
        {
            return History = History with { Remark = value };
        }

        // [ObservableProperty]
        // public partial string? Remark { get; set; }

        
        // [ObservableProperty]
        // public partial bool IsEditing { get; set; }
        //
        [ObservableProperty]
        public partial bool IsInvalid { get; set; }

        [RelayCommand]
        private void Open()
        {
            var msg = new OnAddTab()
            {
                Closable = true,
                Name = History.WorkingCopyPath.GetFileName(),
                Factory = parent => new WorkspaceViewModel(History.WorkingCopyPath, parent)
                {
                    History = History
                }
            };

            SendMessage(msg);
        }

        [ObservableProperty]
        public partial bool IsConflict { get; set; }
        
        [ObservableProperty]
        public partial bool IsModified { get; set; }
        
        [ObservableProperty]
        public partial string Revision { get; set; } = string.Empty;


        private async Task Detect()
        {
            using var context = EngineBackend.Instance.SimpleContext(SendMessage(new OnGetDialogHostId()));
            
            var isModified = false;
            var isConflict = false;
            var receiver = new StatusReceiverDelegate()
            {
                OnStatusEntryAction = entry =>
                {
                    isModified = true;
                    if (entry.Conflicted)
                    {
                        isConflict = true;
                    }
                    if (isModified && isConflict)
                    {
                        throw new CSharpException.SubversionException(SvnErrnoConstants.Default.CeaseInvocationValue(),
                            "Finished");
                    }
                }
            };

            try
            {
                var opts = new StatusOptions(History.WorkingCopyRoot, new Revision.Working(), Depth.Infinity, false,
                    false, false, false, false, false, null);
                await context.StatusNext(opts, receiver);
                IsModified = isModified;
                IsConflict = isConflict;

                var wc = context.WorkingCopyContext();

                var result =
                    await wc.RevsionStatus(new WorkingCopyRevisionStatusOptions(History.WorkingCopyRoot, null, false));

                Revision = result.Status.MinRevision == result.Status.MaxRevision
                    ? $"r{result.Status.MinRevision}"
                    : $"r{result.Status.MinRevision}:r{result.Status.MaxRevision}";
            }
            catch (System.Exception e)
            {
                Console.WriteLine(e);
                IsInvalid = true;
            }
            finally
            {
                SendMessage(new OnHistoryStateChanged());
            }
        }
        
        protected override Task LoadOnce()
        {
            Remark = History.Remark;
            return Detect();
        }

        
    }


    public partial class HistoryGroupItemViewModel(ViewModelBase parent) : ViewModelBase(parent)
    {
        [ObservableProperty]
        public required partial HistoryGroup HistoryGroup { get; set; }
        
        
        public string Name => HistoryGroup.Name;

        public int Count => HistoryGroup.Children.Length;

        [RelayCommand]
        private async Task Delete()
        {
            var hostId = SendMessage(new OnGetDialogHostId());
            var result = await MessageBox.ShowOverlayAsync("Whether to delete", title: "Warning", hostId, MessageBoxIcon.Warning, MessageBoxButton.YesNo);
            if (result != MessageBoxResult.Yes)
            {
                return;
            }
            await EngineBackend.Instance.DatabaseQueue.RunAndWait(async _ =>
            {
                await DatabaseManager.Default.DeleteHistoryGroup(HistoryGroup.Id);
            });
        }
    }
    
    // public partial class HistoryItemViewModel(ViewModelBase parent): ViewModelBase(parent)
    // {
    //     public string Name
    //     {
    //         get
    //         {
    //             return History switch
    //             {
    //                 WorkspaceHistory.Repository => "Repository",
    //                 WorkspaceHistory.WorkingCopy wc => wc.WorkingCopyRoot.GetFileName(),
    //                 _ => string.Empty
    //             };
    //         }
    //     }
    //
    //     public string Path
    //     {
    //         get
    //         {
    //             return History switch
    //             {
    //                 WorkspaceHistory.Repository repository => repository.RootUrl,
    //                 WorkspaceHistory.WorkingCopy wc => wc.WorkingCopyPath,
    //                 _ => string.Empty
    //             };
    //         }
    //     }
    //
    //     public string? RepositoryRootUrl
    //     {
    //         get
    //         {
    //             return History switch
    //             {
    //                 WorkspaceHistory.Repository repository => repository.RootUrl,
    //                 WorkspaceHistory.WorkingCopy wc => wc.RepositoryRootUrl,
    //                 _ => string.Empty
    //             };
    //         }
    //     }
    //
    //     public string? LastUsedTime
    //     {
    //         get
    //         {
    //             return History switch
    //             {
    //                 WorkspaceHistory.Repository repository => repository.LastUsedTime?.Map(seconds =>
    //                 {
    //                     var offset = DateTimeOffset.FromUnixTimeSeconds(seconds);
    //                     return offset.ToLocalTime().DateTime.ToString(CultureInfo.InvariantCulture);
    //                 }),
    //                 WorkspaceHistory.WorkingCopy wc => wc.LastUsedTime?.Map(seconds =>
    //                 {
    //                     var offset = DateTimeOffset.FromUnixTimeSeconds(seconds);
    //                     return offset.ToLocalTime().DateTime.ToString(CultureInfo.InvariantCulture);
    //                 }),
    //                 _ => string.Empty
    //             };
    //         }   
    //     }
    //
    //     public bool IsStar
    //     {
    //         get
    //         {
    //             return History switch
    //             {
    //                 WorkspaceHistory.Repository repository => repository.Star,
    //                 WorkspaceHistory.WorkingCopy wc => wc.Star,
    //                 _ => false
    //             };
    //         }
    //         set
    //         {
    //             History = History.WithIsStar(value);
    //             SetIsStarCommand.Execute(value);
    //         }
    //     }
    //
    //     public List<MenuItemViewModel> MenuItems =>
    //     [
    //         new()
    //         {
    //             Header = "Open",
    //             Command = OpenCommand
    //         },
    //         new()
    //         {
    //             Header = "Delete",
    //             Command = DeleteCommand
    //         },
    //     ];
    //
    //
    //     [RelayCommand]
    //     private void Open()
    //     {
    //         if (History is not WorkspaceHistory.WorkingCopy workingCopy) return;
    //         // var tab = new MainWindowViewModel.TabItemViewModel(GetParent<MainWindowViewModel>()!)
    //         // {
    //         //     Closable = true,
    //         //     Text = workingCopy.WorkingCopyPath.GetFileName(),
    //         // }.Apply(item =>
    //         // {
    //         //     item.Content = new WorkspaceViewModel(workingCopy.WorkingCopyPath, item)
    //         //     {
    //         //         History = workingCopy
    //         //     };
    //         // });
    //         
    //         SendMessage(new OnAddTab
    //         {
    //             Closable = true,
    //             Name = workingCopy.WorkingCopyPath.GetFileName(),
    //             Factory = parent => new WorkspaceViewModel(workingCopy.WorkingCopyPath, parent)
    //         });
    //     }
    //
    //     [RelayCommand]
    //     private async Task Delete()
    //     {
    //         await DatabaseManager.Default.DeleteWorkspaceHistory(History.Uuid);
    //
    //         SendMessage(new OnRemoveHistory()
    //         {
    //             Item = this
    //         });
    //     }
    //
    //     [RelayCommand]
    //     private void ToggleIsStar()
    //     {
    //         IsStar = !IsStar;
    //         Logger.Info("Toggle IsStar is " + IsStar);
    //     }
    //
    //     [RelayCommand(AllowConcurrentExecutions = true)]
    //     private async Task OnSetIsStar(bool value)
    //     {
    //         await EngineBackend.Instance.DatabaseQueue.RunAndWait(async _ =>
    //         {
    //             History = History.WithIsStar(value);
    //             await DatabaseManager.Default.SetWorkspaceHistory(History);
    //         });
    //     }
    //
    //     public async Task Detect()
    //     {
    //         if (History is not WorkspaceHistory.WorkingCopy workingCopy)
    //         {
    //             return;
    //         }
    //         var context = EngineBackend.Instance.SimpleContext(SendMessage(new OnGetDialogHostId()));
    //         
    //         var isModified = false;
    //         var isConflict = false;
    //         var receiver = new StatusReceiverDelegate()
    //         {
    //             OnStatusEntryAction = entry =>
    //             {
    //                 isModified = true;
    //                 if (entry.Conflicted)
    //                 {
    //                     isConflict = true;
    //                 }
    //                 if (isModified && isConflict)
    //                 {
    //                     throw new CSharpException.SubversionException(SvnErrnoConstants.Default.CeaseInvocationValue(),
    //                         "Finished");
    //                 }
    //             }
    //         };
    //
    //         try
    //         {
    //             var opts = new StatusOptions(workingCopy.WorkingCopyRoot, new Revision.Working(), Depth.Infinity, false, false, false, false, false, false, null);
    //             await context.StatusNext(opts, receiver);
    //             IsModified = isModified;
    //             IsConflict = isConflict;
    //         }
    //         catch (System.Exception e)
    //         {
    //             Console.WriteLine(e);
    //         }
    //     }
    //     
    //
    //     [ObservableProperty]
    //     public partial bool IsConflict { get; set; }
    //     
    //     [ObservableProperty]
    //     public partial bool IsModified { get; set; }
    //     
    //     [ObservableProperty]
    //     public partial string Revision { get; set; } = string.Empty;
    //     
    //     public bool IsVisible { get; set; } = true;
    //     
    //     [ObservableProperty]
    //     [NotifyPropertyChangedFor(nameof(IsStar))]
    //     public required partial WorkspaceHistory History { get; set; }
    //
    //     [RelayCommand]
    //     private async Task OnLoaded()
    //     {
    //         if (History is WorkspaceHistory.WorkingCopy workingCopy)
    //         {
    //             
    //             var opts = new WorkingCopyCreateContextOptions();
    //             var context = AsyncWorkingCopyContext.Create(opts);
    //         
    //             var result = await context.RevsionStatus(new WorkingCopyRevisionStatusOptions(workingCopy.WorkingCopyRoot, null, false));
    //
    //             Revision = result.Status.MinRevision == result.Status.MaxRevision ? $"r{result.Status.MinRevision}" : $"r{result.Status.MinRevision}:r{result.Status.MaxRevision}";
    //         }
    //         
    //     }
    // }
    //
    
    //
    // [ObservableProperty]
    // public partial object? DetailViewModel { get; set; }
    
    
    [ObservableProperty]
    public partial ObservableCollection<HistoryItemViewModel> HistoryItems { get; set; } = [];

    [ObservableProperty] public partial int SelectedKindIndex { get; set; } = AllIndex;

    [ObservableProperty] public partial int SelectedGroupIndex { get; set; } = -1;

    [ObservableProperty] public partial string SearchText { get; set; } = string.Empty;


    [ObservableProperty]
    public partial ObservableCollection<HistoryGroupItemViewModel> HistoryGroupItemViewModels
    {
        get;
        set;
    } = [];

    public int InvalidCount => HistoryItems.Count(i => i is WorkingCopyItemViewModel { IsInvalid: true });

    public int ModifiedCount => HistoryItems.Count(i => i is WorkingCopyItemViewModel { IsModified: true });
    
    public int ConflictCount => HistoryItems.Count(i => i is WorkingCopyItemViewModel { IsConflict: true });
    
    public int StarCount => HistoryItems.Count(i => i.IsStar);
    
    [ObservableProperty]
    public partial HistoryItemViewModel? SelectedHistoryItem { get; set; }

    partial void OnSelectedKindIndexChanged(int value)
    {
        if (value < 0)
        {
            return;
        }

        SelectedGroupIndex = -1;
        ApplyFilter();
    }

    

    partial void OnSelectedGroupIndexChanged(int value)
    {
        if (value < 0)
        {
            return;
        }

        SelectedKindIndex = -1;
        ApplyFilter();
    }


    private void ApplyFilter()
    {
        if (SelectedKindIndex >= 0)
        {
            switch (SelectedKindIndex)
            {
                case AllIndex:
                    foreach (var historyItemViewModel in HistoryItems)
                    {
                        historyItemViewModel.IsVisible = true;
                    }
                    break;
                case ModifiedIndex:
                    foreach (var historyItemViewModel in HistoryItems)
                    {
                        historyItemViewModel.IsVisible = historyItemViewModel is WorkingCopyItemViewModel { IsModified: true };
                    }
                    break;
                case ConflictedIndex:
                    foreach (var historyItemViewModel in HistoryItems) 
                    {
                        historyItemViewModel.IsVisible = historyItemViewModel is WorkingCopyItemViewModel { IsConflict: true };
                    }
                    break;
                case InvalidIndex:
                    foreach (var historyItemViewModel in HistoryItems)
                    {
                        historyItemViewModel.IsVisible = historyItemViewModel is WorkingCopyItemViewModel { IsInvalid: true };
                    }
                    break;
                case StarIndex:
                    foreach (var historyItemViewModel in HistoryItems)
                    {
                        historyItemViewModel.IsVisible = historyItemViewModel.IsStar;
                    }
                    break;
            }
        }
        else if (SelectedGroupIndex >= 0 && SelectedGroupIndex < HistoryGroupItemViewModels.Count)
        {
            var group = HistoryGroupItemViewModels[SelectedGroupIndex];
            foreach (var historyItemViewModel in HistoryItems)
            {
                historyItemViewModel.IsVisible = group.HistoryGroup.Children.Contains(historyItemViewModel.HistoryUuid);
            }
        }


        if (SelectedHistoryItem is not null && !SelectedHistoryItem.IsVisible)
        {
            SelectedHistoryItem = null;
        }
    }

    [RelayCommand]
    private async Task AddHistoryGroup()
    {
        var hostId = SendMessage(new OnGetDialogHostId());

        var dialogOptions = new OverlayDialogOptions()
        {
            IsCloseButtonVisible = true,
            Title = "Add History Group",
            StyleClass = "Fixed",
            Buttons = DialogButton.None,
        };

        var model = new AddHistoryGroupDialogModel();
        await OverlayDialog.ShowModal<AddHistoryGroupDialog, AddHistoryGroupDialogModel>(model, hostId, dialogOptions);

        if (model.HistoryGroup is not null)
        {
            HistoryGroupItemViewModels.Add(new HistoryGroupItemViewModel(this)
            {
                HistoryGroup = model.HistoryGroup,
            });
        }
    }

    [RelayCommand]
    private async Task OnLoaded()
    {
        await EngineBackend.Instance.DatabaseQueue.Run(async token =>
        {
            var historyItems = await DatabaseManager.Default.WorkspaceHistories();
        
            HistoryItems = new ObservableCollection<HistoryItemViewModel>(historyItems.Select<WorkspaceHistory, HistoryItemViewModel>(i =>
            {
                return i switch
                {
                    WorkspaceHistory.WorkingCopy workingCopy => new WorkingCopyItemViewModel(this)
                    {
                        History = workingCopy,
                    },
                    WorkspaceHistory.Repository repository =>
                        new RepositoryItemViewModel(this) { History = repository },
                    _ => throw new UnreachableException()
                };
            }));

            var historyGroups = await DatabaseManager.Default.HistoryGroups();
            HistoryGroupItemViewModels = new ObservableCollection<HistoryGroupItemViewModel>(historyGroups.Select<HistoryGroup, HistoryGroupItemViewModel>(i => new HistoryGroupItemViewModel(this)
            {
                HistoryGroup = i
            }));
            // foreach (var item in HistoryItems)
            // { 
            //     _ = Dispatcher.UIThread.InvokeAsync(async () =>
            //     {
            //         await item.Detect();
            //     });
            // }
        });
    }
    
    
    [RelayCommand]
    private async Task ShowCheckoutDialog()
    {
        var options = new OverlayDialogOptions
        {
            Title = "Checkout",
            IsCloseButtonVisible = true,
            Buttons = DialogButton.None
        };

        var hostId = SendMessage(new OnGetDialogHostId());

        var model = new CheckoutOrExportDialogModel();
        
        await OverlayDialog.ShowModal<CheckoutOrExportDialog, CheckoutOrExportDialogModel>(model, hostId, options: options);
        if (model.Options is not null)
        {
            Receive(model.Options);
        }
    }


    [RelayCommand]
    private async Task OpenLocalRepository()
    {
        var options = new FolderPickerOpenOptions()
        {
            AllowMultiple = false,
            Title = "Select a local repository",
        };
        
        
        var result = await Manager.Default.Send(new OnFolderPickerOpen(options), Manager.MainWindowToken);
        if (result.Count == 1)
        {
            
            var path = result[0].Path.AbsolutePath.TrimEndPathSeparatorChar();


            // var tab = new MainWindowViewModel.TabItemViewModel(GetParent<MainWindowViewModel>()!)
            // {
            //     Closable = true,
            //     Text = result[0].Name
            // }.Apply(item =>
            // {
            //     item.Content = new WorkspaceViewModel(path, item);
            // });
            
            SendMessage(new OnAddTab
            {
                Closable = true,
                Name = result[0].Name,
                Factory = parent => new WorkspaceViewModel(path, parent),
            });
        }
    }

    public void Receive(CheckoutOptions message)
    {
        var hostId = SendMessage(new OnGetDialogHostId());
        var messenger = new WeakReferenceMessenger();
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.Url,
            Path = message.Path,
            Messenger = messenger
        };
        


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStart(message.Path).ToString();
                    model.ProcessLogItems.Add(new CheckoutOrExportProcessDialogModel.ProcessLogItemViewModel()
                    {
                        Action = notify.Action.ToString(),
                        MimeType =  notify.MimeType ?? "",
                        Path = path
                    });
                    model.CurrentFile = path;
                });
            },
            ProgressNotifyAction = (downloaded, total) =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    model.Total = total;
                    model.Downloaded = downloaded;
                });
            },
            DialogHostId = hostId
        };
        
        
        var createContextOptions = EngineBackend.Instance.MakeCreateContextOptions(contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        messenger.Register<OnCancel>(contextNotifier, (recipient, cancel) =>
        {
            (recipient as ContextNotifierDelegate)!.CancelMessage = "User cancel";
        });

        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Checkout",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };
            Console.WriteLine("Show dialog");
            await OverlayDialog.ShowModal<CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: hostId, options: options);
        });
        
        Task.Run(async () =>
        {
            try
            {
        
                Console.WriteLine("Checkout now");
                await context.Checkout(message);
                Console.WriteLine("Checkout now finished");
                Dispatcher.UIThread.Invoke(() => { model.IsCompleted = true; });
            }
            catch (System.Exception e)
            {
                model.Error = e.HumanReadableMessage;
            }
            finally
            {
                context.Dispose();
            }
        });
        
        
        


    }


    [RelayCommand]
    private async Task ShowExportDialog()
    {
        var exportDialogModel = new ExportDialogModel();
        
        var options = new OverlayDialogOptions
        {
            Title = "Export",
            IsCloseButtonVisible = true,
            Buttons = DialogButton.None
        };
        
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId());
        
        await OverlayDialog.ShowModal<ExportDialog, ExportDialogModel>(exportDialogModel, options: options, hostId: hostId);
        if (exportDialogModel.Options is not null)
        {
            Receive(exportDialogModel.Options);
        }
        
    }

    public void Receive(ExportOptions message)
    {
        
        // var hostId = Manager.Default.Send(new OnGetDialogHostId(), _tabService.Token).Response;
        var hostId = SendMessage(new OnGetDialogHostId());
        var messenger = new WeakReferenceMessenger();
        var model = new CheckoutOrExportProcessDialogModel
        {
            Url = message.FromPathOrUrl,
            Path = message.ToPath,
            Messenger = messenger
        };
        


        var contextNotifier = new ContextNotifierDelegate()
        {
            WorkingCopyNotifyAction = notify =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    var path = notify.Path.TrimStart(message.ToPath).ToString();
                    model.ProcessLogItems.Add(new CheckoutOrExportProcessDialogModel.ProcessLogItemViewModel()
                    {
                        Action = notify.Action.ToString(),
                        MimeType =  notify.MimeType ?? "",
                        Path = path
                    });
                    model.CurrentFile = path;
                });
            },
            ProgressNotifyAction = (downloaded, total) =>
            {
                Dispatcher.UIThread.Invoke(() =>
                {
                    model.Total = total;
                    model.Downloaded = downloaded;
                });
            },
            DialogHostId = hostId
        };
        
        
        var createContextOptions = EngineBackend.Instance.MakeCreateContextOptions(contextNotifier);

        var context = AsyncContext.Create(createContextOptions);
        
        messenger.Register<OnCancel>(contextNotifier, (recipient, cancel) =>
        {
            (recipient as ContextNotifierDelegate)!.CancelMessage = "User cancel";
        });

        Dispatcher.UIThread.InvokeAsync(async () =>
        {
            var options = new OverlayDialogOptions
            {
                Title = "Export",
                IsCloseButtonVisible = false,
                Buttons = DialogButton.None
            };
            Console.WriteLine("Show dialog");
            await OverlayDialog.ShowModal<CheckoutOrExportProcessDialog, CheckoutOrExportProcessDialogModel>(model, hostId: hostId, options: options);
        });
        
        Task.Run(async () =>
        {
            try
            {
                await context.Export(message);
                Dispatcher.UIThread.Invoke(() => { model.IsCompleted = true; });
            }
            catch (System.Exception e)
            {
                model.Error = e.HumanReadableMessage;
            }
            finally
            {
                context.Dispose();
            }
        });
        
    }

    public void Receive(OnRemoveHistory message)
    {
        HistoryItems.Remove(message.Item);
    }

    public void Receive(OnHistoryStateChanged message)
    {
        OnPropertyChanged(nameof(InvalidCount));
        OnPropertyChanged(nameof(ModifiedCount));
        OnPropertyChanged(nameof(ConflictCount));
        OnPropertyChanged(nameof(StarCount));
        ApplyFilter();
    }
}