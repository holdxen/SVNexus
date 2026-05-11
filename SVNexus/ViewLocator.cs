using System;
using System.Collections.Generic;
using Avalonia.Controls;
using Avalonia.Controls.Templates;
using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.ViewModels.WorkingCopy.Changes;
using SVNexus.ViewModels.WorkingCopy.History;
using SVNexus.ViewModels.WorkingCopy.Local;
using SVNexus.ViewModels.WorkingCopy.Remote;
using SVNexus.Views;
using SVNexus.Views.WorkingCopy;
using SVNexus.Views.WorkingCopy.Changes;
using SVNexus.Views.WorkingCopy.History;
using SVNexus.Views.WorkingCopy.Local;
using SVNexus.Views.WorkingCopy.Remote;

namespace SVNexus;

/// <summary>
/// Given a view model, returns the corresponding view without using reflection.
/// AOT-safe: all ViewModel→View mappings are registered explicitly at startup.
/// </summary>
public class ViewLocator : IDataTemplate
{
    /// <summary>
    /// Maps a ViewModel <see cref="Type"/> to a factory that creates the corresponding View.
    /// Populated once in the static constructor — no reflection, fully AOT-compatible.
    /// </summary>
    private static readonly Dictionary<Type, Func<Control>> Registry = new()
    {
        // ── Top-level mappings ──
        [typeof(MainWindowViewModel)]                = () => new MainWindow(),
        [typeof(WelcomeViewModel)]                   = () => new WelcomeView(),
        [typeof(SvgIconViewModel)]                   = () => new SvgIconView(),
        [typeof(DifferenceViewModel)]                = () => new DifferenceView(),

        // Dialogs
        [typeof(AuthenticateDialogModel)]            = () => new AuthenticateDialog(),
        [typeof(CheckoutOrExportDialogModel)]        = () => new CheckoutOrExportDialog(),
        [typeof(CommitDialogModel)]                  = () => new CommitDialog(),
        [typeof(DeleteDialogModel)]                  = () => new DeleteDialog(),
        [typeof(DifferenceDialogModel)]              = () => new DifferenceDialog(),
        [typeof(ExportDialogModel)]                  = () => new ExportDialog(),
        [typeof(ImportDialogModel)]                  = () => new ImportDialog(),
        [typeof(ImportProcessDialogModel)]           = () => new ImportProcessDialog(),
        [typeof(InfoDialogModel)]                    = () => new InfoDialog(),
        [typeof(InitializeRepositoryDialogModel)]    = () => new InitializeRepositoryDialog(),
        [typeof(InitializeRepositoryProcessDialogModel)] = () => new InitializeRepositoryProcessDialog(),
        [typeof(LockDialogModel)]                    = () => new LockDialog(),
        [typeof(MkdirDialogModel)]                   = () => new MkdirDialog(),
        [typeof(PatchDialogModel)]                   = () => new PatchDialog(),
        [typeof(RevertDialogModel)]                  = () => new RevertDialog(),
        [typeof(TrustServerDialogModel)]             = () => new TrustServerDialog(),
        [typeof(UnlockDialogModel)]                  = () => new UnlockDialog(),
        [typeof(UpdateDialogModel)]                  = () => new UpdateDialog(),
        [typeof(AddHistoryGroupDialogModel)]         = () => new AddHistoryGroupDialog(),

        // ── WorkingCopy ──
        [typeof(WorkingCopyViewModel)]               = () => new WorkingCopyView(),
        [typeof(WorkspaceViewModel)]                 = () => new WorkspaceView(),
        [typeof(TargetItemViewModel)]                = () => new TargetItemView(),

        // ── Changes ──
        [typeof(ChangesViewModel)]                   = () => new ChangesView(),
        [typeof(ChangesListViewModel)]               = () => new ChangesListView(),
        [typeof(ChangesTreeViewModel)]               = () => new ChangesTreeView(),

        // ── History ──
        [typeof(HistoryViewModel)]                   = () => new HistoryView(),
        [typeof(HistoryDetailViewModel)]             = () => new HistoryDetailView(),
        [typeof(HistoryChangesViewModel)]            = () => new HistoryChangesView(),
        [typeof(HistorySnapshotViewModel)]           = () => new HistorySnapshotView(),

        // ── Local / Remote ──
        [typeof(LocalViewModel)]                     = () => new LocalView(),
        [typeof(RemoteViewModel)]                    = () => new RemoteView(),
    };

    public Control? Build(object? param)
    {
        switch (param)
        {
            case null:
                return null;
            // 1. If the ViewModel specifies a LocateType, use that to find the view.
            //    (e.g. ChangesListViewModel.ListItemViewModel → TargetItemViewModel → TargetItemView)
            case ViewModelBase { LocateType: not null } vm when Registry.TryGetValue(vm.LocateType, out var locateFactory):
                return locateFactory();
        }

        // 2. Look up the exact runtime type.
        var type = param.GetType();
        if (Registry.TryGetValue(type, out var factory))
        {
            return factory();
        }

        // 3. No mapping found — return a placeholder.
        return new TextBlock { Text = "Not Found For Model: " + type.FullName };
    }

    public bool Match(object? data)
    {
        return data is ViewModelBase or ViewModelLite;
    }
}
