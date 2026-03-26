using System;
using Microsoft.Extensions.DependencyInjection;
using SVNexus.Inject;
using SVNexus.Messages;
using SVNexus.Utils;

namespace SVNexus.ViewModels.WorkingCopy;

public class WorkspaceViewModel: ViewModelLite
{
    public class ScopeContent
    {
        public IServiceScope? Scope { get; set; }
        public required WorkingCopyViewModel WorkingCopyViewModel { get; set; }
    }

    private readonly LimitedDictionary<string, ScopeContent> _workingCopyViewModels = new()
    {
        Limit = 20
    };
    
    private readonly Services.IWorkingCopyViewService _workingCopyViewService;
    
    private readonly Services.TypeService _typeService;
    
    public WorkingCopyViewModel WorkingCopyViewModel { get; }
    
    public WorkspaceViewModel(IServiceProvider serviceProvider)
    {
        _workingCopyViewService = serviceProvider.GetRequiredService<Services.IWorkingCopyViewService>();
        _typeService = serviceProvider.GetRequiredService<Services.TypeService>();
        
        WorkingCopyViewModel = new WorkingCopyViewModel(serviceProvider);
        
        Manager.Default.RegisterAllMessages(this, _typeService.Get(this));
    }
    
    
}