using System;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using Irihi.Avalonia.Shared.Contracts;
using SVNexus.Generated;
using SVNexus.Views;

namespace SVNexus.ViewModels;



public partial class TrustServerDialogModel: ViewModelBase, IDialogContext
{
    public partial class AsciiCertModel: ViewModelBase
    {
        [ObservableProperty]
        public partial string? AsciiCert { get; set; }
    }
    
    public AsciiCertModel AsciiCertViewModel { get; set; } = new();
    
    public override Type? ViewType { get; set; } = typeof(TrustServerDialog);
    
    
    [ObservableProperty]
    public partial string? Realm { get; set; }
    
    [ObservableProperty]
    public partial string? Hostname { get; set; }
    
    [ObservableProperty]
    public partial string? Fingerprint { get; set; }
    
    
    [ObservableProperty]
    public partial string? ValidFrom { get; set; }
    
    
    [ObservableProperty]
    public partial string? ValidUntil { get; set; }
    
    
    [ObservableProperty]
    public partial string? Issuer { get; set; }
    
    
    
    [ObservableProperty]
    public partial string? AsciiCert { get; set; }
    
    
    [ObservableProperty]
    public partial bool Savable { get; set; }
    
    
    [ObservableProperty]
    public partial bool Save { get; set; }
    
    
    public bool Accept { get; set; }

    [RelayCommand]
    public void Close()
    {
        RequestClose?.Invoke(this, null);
        Accept = false;
    }

    public event EventHandler<object?>? RequestClose;


    [RelayCommand]
    public void Confirm()
    {
        Accept = true;
        RequestClose?.Invoke(this, null);
    }
}