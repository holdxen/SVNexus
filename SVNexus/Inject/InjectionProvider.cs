// using System;
// using Jab;
// using SVNexus.ViewModels;
// using SVNexus.ViewModels.WorkingCopy;
//
// namespace SVNexus.Inject;
//
// [ServiceProvider]
// [Singleton(typeof(InjectionProvider), Factory = nameof(GetSelf))]
// [Singleton(typeof(ITabContext), Factory = nameof(GetTabContext))]
// [Singleton(typeof(IWorkingCopyViewContext), Factory =  nameof(GetWorkingCopyContext))]
// public partial class InjectionProvider(
//     IWorkingCopyViewContext? workingCopyContext = null, 
//     IWelcomeViewContext? welcomeViewContext = null)
// {
//     private InjectionProvider GetSelf()
//     {
//         return this;
//     }
//
//     private ITabContext GetTabContext()
//     {
//         return _tabContext;
//     }
//
//     private IWorkingCopyViewContext GetWorkingCopyContext()
//     {
//         return workingCopyContext ?? throw new Exception($"{nameof(workingCopyContext)} cannot be null.");
//     }
//
//     private IWelcomeViewContext GetWelcomeViewContext()
//     {
//         return welcomeViewContext ?? throw new Exception($"{nameof(welcomeViewContext)} cannot be null.");
//     }
//     
//
//     private readonly ITabContext _tabContext = new TabContext();
// }

using SVNexus.ViewModels;
using SVNexus.ViewModels.WorkingCopy;
using SVNexus.ViewModels.WorkingCopy.Changes;
using SVNexus.ViewModels.WorkingCopy.History;
using SVNexus.ViewModels.WorkingCopy.Local;

namespace SVNexus.Inject;

public static class InjectionProvider
{
    // public static ServiceProvider Provider { get; private set; } = null!;
    //
    // public static void Initialize()
    // {
    //     var builder = new ServiceCollection();
    //
    //     builder.AddScoped<Services.ITabService, Services.TabService>();
    //     builder.AddScoped<Services.IWelcomeViewService, Services.WelcomeViewService>();
    //     
    //     builder.AddScoped<Services.WorkingCopyViewService>();
    //     
    //     builder.AddScoped<Services.IWorkingCopyViewService>(sp => sp.GetRequiredService<Services.WorkingCopyViewService>());
    //     
    //     builder.AddScoped<Services.EmptyTokenService>();
    //     
    //     builder.AddScoped<Services.ITokenService>(sp => sp.GetRequiredService<Services.EmptyTokenService>());
    //     builder.AddScoped<Services.TypeService>();
    //
    //     builder.AddTransient<ChangesListViewModel>();
    //     builder.AddTransient<ChangesTreeViewModel>();
    //     builder.AddTransient<ChangesViewModel>();
    //     builder.AddTransient<HistoryChangesViewModel>();
    //     builder.AddTransient<HistoryDetailViewModel>();
    //     builder.AddTransient<HistorySnapshotViewModel>();
    //     builder.AddTransient<HistoryViewModel>();
    //     builder.AddTransient<LocalViewModel>();
    //     builder.AddTransient<WorkingCopyViewModel>();
    //     builder.AddTransient<WelcomeViewModel>();
    //     builder.AddTransient<WorkspaceViewModel>();
    //     
    //     Provider = builder.BuildServiceProvider();
    // }
    //
    // public static IContainer Container { get; set; } = null!;
    //
    // public static void Setup()
    // {
    //     var builder = new ContainerBuilder();
    //
    //     builder.RegisterAssemblyTypes(typeof(InjectionProvider).Assembly)
    //         .PublicOnly()
    //         .Where(t => t.Namespace?.StartsWith("SVNexus.ViewModels") ?? false)
    //         .AsSelf()
    //         .InstancePerDependency();
    //     
    //     Container = builder.Build();
    // }
}