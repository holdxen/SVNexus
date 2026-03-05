using System.Diagnostics;
using System.IO;
using System.Text;
using Avalonia.Rendering;
using Serilog;
using Avalonia.Controls.Mixins;
using Avalonia.Svg.Skia;
using Svg.Skia;

namespace SVNexus.Components;




using System;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Avalonia.Threading;
using Generated;

/// <summary>
/// 根据像素尺寸调用 RGBA 生成器并显示的控件。
/// </summary>
public class SvgRender : UserControl
{
    // 你的生成器：输入像素宽高，返回长度 = w*h*4 的 RGBA(未预乘) 或预乘数组
    // public static readonly StyledProperty<Func<int, int, byte[]>?> DataFactoryProperty =
    //     AvaloniaProperty.Register<RgbaGeneratedView, Func<int, int, byte[]>?>(nameof(DataFactory));
    //
    // public Func<int, int, byte[]>? DataFactory
    // {
    //     get => GetValue(DataFactoryProperty);
    //     set => SetValue(DataFactoryProperty, value);
    // }

    static SvgRender()
    {
        PressedMixin.Attach<SvgRender>();
    }
    
    public static readonly StyledProperty<bool> LoadBuiltinFontsProperty = AvaloniaProperty.Register<SvgRender, bool>(
        nameof(LoadBuiltinFonts), defaultValue: true);

    public bool LoadBuiltinFonts
    {
        get => GetValue(LoadBuiltinFontsProperty);
        set => SetValue(LoadBuiltinFontsProperty, value);
    }
    
    public static readonly StyledProperty<string?> SourceProperty =
        AvaloniaProperty.Register<SvgRender, string?>(nameof(Source));


    public string? Source
    {
        get => GetValue(SourceProperty);
        set => SetValue(SourceProperty, value);
    }

    // 源数据是否为预乘 Alpha
    public static readonly StyledProperty<bool> PremultipliedProperty =
        AvaloniaProperty.Register<SvgRender, bool>(nameof(Premultiplied), false);

    public bool Premultiplied
    {
        get => GetValue(PremultipliedProperty);
        set => SetValue(PremultipliedProperty, value);
    }

    // 源数据是否为 BGRA（否则视为 RGBA）
    public static readonly StyledProperty<bool> SourceIsBgraProperty =
        AvaloniaProperty.Register<SvgRender, bool>(nameof(SourceIsBgra), false);

    public bool SourceIsBgra
    {
        get => GetValue(SourceIsBgraProperty);
        set => SetValue(SourceIsBgraProperty, value);
    }

    // 尺寸变化去抖间隔（毫秒）。设为 0 关闭去抖。
    public static readonly StyledProperty<int> DebounceMillisecondsProperty =
        AvaloniaProperty.Register<SvgRender, int>(nameof(DebounceMilliseconds), 50);

    public int DebounceMilliseconds
    {
        get => GetValue(DebounceMillisecondsProperty);
        set => SetValue(DebounceMillisecondsProperty, value);
    }

    private readonly Image _image = new()
    {
        Stretch = Stretch.Fill
    };

    private WriteableBitmap? _wb;
    private IDisposable? _boundsSub;
    private int _pxW, _pxH;
    private DispatcherTimer? _debounceTimer;
    private bool _pending;


    public SvgRender()
    {
        Content = _image;
        // this.GetObservable(DataFactoryProperty).Subscribe(_ => Regenerate(force: true));
        this.GetObservable(SourceProperty).Subscribe(_ => Regenerate(force: true));
        this.GetObservable(PremultipliedProperty).Subscribe(_ => Regenerate(force: true));
        this.GetObservable(SourceIsBgraProperty).Subscribe(_ => Regenerate(force: true));
        this.GetObservable(DebounceMillisecondsProperty).Subscribe(_ =>
        {
            _debounceTimer?.Stop();
            _debounceTimer = null; // 下次用到再重建
        });
        // this.GetObservable(DataFactoryProperty).Subscribe();
    }


    // protected override Size MeasureOverride(Size availableSize)
    // {
    //     if (double.IsInfinity(availableSize.Width) || double.IsInfinity(availableSize.Height))
    //         return new Size(100, 100);
    //     
    //     var size = availableSize.Height > availableSize.Width ? availableSize.Height : availableSize.Width;
    //     Log.Information("Measure render complete: {size}", size);
    //     return  new Size(size, size);
    // }

    protected override void OnAttachedToVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnAttachedToVisualTree(e);
        _boundsSub = this.GetObservable(BoundsProperty).Subscribe(_ => OnBoundsChanged());
        Regenerate(force: true);
    }

    protected override void OnDetachedFromVisualTree(VisualTreeAttachmentEventArgs e)
    {
        base.OnDetachedFromVisualTree(e);
        _boundsSub?.Dispose();
        _boundsSub = null;
        _image.Source = null;
        _wb?.Dispose();
        _wb = null;
    }

    private void OnBoundsChanged()
    {
        var ms = DebounceMilliseconds;
        if (ms <= 0)
        {
            Regenerate();
            return;
        }

        _pending = true;
        _debounceTimer ??= new DispatcherTimer { Interval = TimeSpan.FromMilliseconds(ms) };
        _debounceTimer.Tick -= DebounceTimerOnTick;
        _debounceTimer.Tick += DebounceTimerOnTick;
        _debounceTimer.Stop();
        _debounceTimer.Start();
    }

    private void DebounceTimerOnTick(object? sender, EventArgs e)
    {
        _debounceTimer?.Stop();
        if (!_pending) return;
        _pending = false;
        Regenerate();
    }

    private void Regenerate(bool force = false)
    {
        try
        {

            if (string.IsNullOrWhiteSpace(Source) || Bounds.Width <= 0 || Bounds.Height <= 0)
            {
                _image.Source = null;
                return;
            }


            // 把 DIP 转成像素尺寸（考虑缩放）
            var scale = VisualRoot?.RenderScaling ?? 1.0;
            var w = Math.Max(1, (int)Math.Round(Bounds.Width * scale));
            var h = Math.Max(1, (int)Math.Round(Bounds.Height * scale));

            if (!force && w == _pxW && h == _pxH) return;
            _pxW = w; _pxH = h;
        

            // 生成 RGBA/BGRA 数据
            var options = new SvgRenderOptions(
                Svg: Source, Width: Convert.ToUInt32(w), Height: Convert.ToUInt32(h));
            var bytes = options.Render();
        

            if (bytes.Length < w * h * 4) return;
        
            var size = new PixelSize(w, h);
            var alpha = Premultiplied ? AlphaFormat.Premul : AlphaFormat.Unpremul;
            var fmt = SourceIsBgra ? PixelFormat.Bgra8888 : PixelFormat.Rgba8888;
        
            if (_wb == null || _wb.PixelSize != size || _wb.Format != fmt || _wb.AlphaFormat != alpha)
            {
                _wb?.Dispose();
                _wb = new WriteableBitmap(size, new Vector(96, 96), fmt, alpha);
            }
        
            using (var fb = _wb.Lock())
            {
                var srcStride = w * 4;
                for (var y = 0; y < h; y++)
                {
                    var dest = fb.Address + y * fb.RowBytes;
                    var srcOff = y * srcStride;
                    Marshal.Copy(bytes, srcOff, dest, srcStride);
                }
            }
        
            _image.Source = _wb;
        }
        catch (System.Exception e)
        {
            Console.WriteLine(e);
            Console.WriteLine("Source:\n{0}", Source?.Trim());
            throw;
        }
    }
}