using System;
using System.Collections.Generic;
using CommunityToolkit.Mvvm.ComponentModel;

namespace SVNexus.Inject;

public static class Services
{

    extension<T>(T t)
    {
        public Guid GetToken(TypeService typeService)
        {
            return typeService.Get<T>();
        }
    }
    
    public class TypeService
    {
        private readonly Dictionary<Type, Guid> _token = [];

        public Guid Get(object obj)
        {
            return Get(obj.GetType());
        }

        public Guid Get(Type type)
        {
            if (_token.TryGetValue(type, out var token))
            {
                return token;
            }

            token = Guid.NewGuid();
            _token[type] = token;
            return token;
        }

        public Guid Get<T>()
        {
            return Get(typeof(T));
        }
    }
    
    
    public class TabService : ITabService
    {
        public Guid Token { get; } = Guid.NewGuid();
    }

    public class WorkingCopyViewService : IWorkingCopyViewService
    {
        public string WorkingCopyPath { get; set; } = string.Empty;
        // public Guid WorkingCopyToken { get; } = Guid.NewGuid();
        // public Guid LocalViewToken { get; } = Guid.NewGuid();
        // public Guid ChangesViewToken { get; } = Guid.NewGuid();
        // public Guid HistoryViewToken { get; } = Guid.NewGuid();
        // public Guid RemoteViewToken { get; } = Guid.NewGuid();
        // public Guid ChangesListViewToken { get; } = Guid.NewGuid();
        // public Guid ChangesTreeViewToken { get; } = Guid.NewGuid();
        // public Guid LocalTreeViewToken { get; } = Guid.NewGuid();
    }

    public class WelcomeViewService: IWelcomeViewService
    {
        public Guid Token { get; } = Guid.NewGuid();
    }

    public interface IWelcomeViewService
    {
        public Guid Token { get; }
    }

    public interface ITabService
    {
        public Guid Token { get; }
    }

    public interface IWorkingCopyViewService
    {
        public string WorkingCopyPath { get; }
    
        // public Guid WorkingCopyToken { get; }
        //
        // public Guid LocalViewToken { get; }
        //
        // public Guid ChangesViewToken { get; }
        //
        // public Guid HistoryViewToken { get; }
        //
        // public Guid RemoteViewToken { get; }
        //
        // public Guid ChangesListViewToken { get; }
        //
        // public Guid ChangesTreeViewToken { get; }
        //
        // public Guid LocalTreeViewToken { get; }
    
    }
    
    public interface ITokenService
    {
        public Guid Token { get; }
    }
    
    public class EmptyTokenService: ITokenService
    {
        public Guid Token { get; set; } = Guid.Empty;
    }

    public class WorkspaceViewService(string path)
    {
        public string Path { get; } = path;
    }
    
    public class WcViewService(string path)
    {
        public string Path { get; } = path;
    }
}