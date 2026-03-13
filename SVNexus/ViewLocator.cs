using System;
using System.Collections.Generic;
using System.Diagnostics.CodeAnalysis;
using System.Linq;
using Avalonia.Controls;
using Avalonia.Controls.Templates;
using SVNexus.Utils;
using SVNexus.ViewModels;

namespace SVNexus;

/// <summary>
/// Given a view model, returns the corresponding view if possible.
/// </summary>
[RequiresUnreferencedCode(
    "Default implementation of ViewLocator involves reflection which may be trimmed away.",
    Url = "https://docs.avaloniaui.net/docs/concepts/view-locator")]
public class ViewLocator : IDataTemplate
{

    private readonly LimitedDictionary<object, Control> _cache = new()
    {
        Limit = 20
    };





    // void CacheControl(object value, Control control)
    // {
    //     const int maxCacheItem = 20;
    //     if (_cache.Count >= maxCacheItem + 1)
    //     {
    //         _cache.Remove(_cache.First().Key);
    //     }
    //     _cache[value] = control;
    // }
    
    public Control? Build(object? param)
    {
        if (param is null)
            return null;

        if (_cache.TryGetValue(param, out var control))
        {
            Console.WriteLine("hit cache for view");
            return control;
        }
        Console.WriteLine("create for view");


        var keepAlive = false;
        Type? viewType = null;
        if (param is ViewModelBase vm)
        {
            viewType = vm.ViewType;
            keepAlive = vm.KeepAlive;
        }

        if (viewType is null)
        {
            var name = param.GetType().FullName!.Replace("ViewModel", "View");

            viewType = Type.GetType(name);
        }
        
        
        
        
        

        if (viewType == null) return new TextBlock { Text = "Not Found For Model: " + param.GetType().FullName };
        
        
        
        var widget = (Control)Activator.CreateInstance(viewType)!;
        
        if (keepAlive)
        {
            _cache.Add(param, widget);
        }

        return widget;
    }

    public bool Match(object? data)
    {
        return data is ViewModelBase or ViewModelLite;
    }
}