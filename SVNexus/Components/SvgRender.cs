using SVNexus.Extension;

namespace SVNexus.Components;




using System;
using System.Runtime.InteropServices;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Media;
using Avalonia.Media.Imaging;
using Avalonia.Platform;
using Generated;

/// <summary>
/// 根据像素尺寸调用 RGBA 生成器并显示的控件。
/// </summary>
public class SvgRender : Control
{
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

    public static readonly StyledProperty<IBrush?> ForegroundProperty = AvaloniaProperty.Register<SvgRender, IBrush?>(
        nameof(Foreground));

    public IBrush? Foreground
    {
        get => GetValue(ForegroundProperty);
        set => SetValue(ForegroundProperty, value);
    }


    
    public static readonly StyledProperty<double> SizeProperty =
        AvaloniaProperty.Register<SvgRender, double>(nameof(Size), 24);
    
    public double Size
    {
        get => GetValue(SizeProperty);
        set => SetValue(SizeProperty, value);
    }

    private WriteableBitmap? _wb;

    static SvgRender()
    {
        AffectsRender<SvgRender>(SourceProperty, SizeProperty, ForegroundProperty);

        AffectsMeasure<SvgRender>(SizeProperty);
    }

    public override void Render(DrawingContext context)
    {
        try
        {
            if (string.IsNullOrWhiteSpace(Source) || Bounds.Width <= 0 || Bounds.Height <= 0)
            {
                return;
            }


            // 把 DIP 转成像素尺寸（考虑缩放）
            var scale = TopLevel.GetTopLevel(this)?.RenderScaling ?? 1.0;
            var w = Math.Max(1, (int)Math.Round(Bounds.Width * scale));
            var h = Math.Max(1, (int)Math.Round(Bounds.Height * scale));


            var hexColor = Foreground switch
            {
                SolidColorBrush brush => brush.Color.ToCssHex(),
                IImmutableSolidColorBrush immutableSolidColorBrush => immutableSolidColorBrush.Color.ToCssHex(),
                _ => null
            };

            // 生成 RGBA/BGRA 数据
            var options = new SvgRenderOptions(
                Svg: Source, Width: Convert.ToUInt32(w), Height: Convert.ToUInt32(h), Color: hexColor);
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
            
            context.DrawImage(_wb, new Rect(0, 0, Bounds.Width, Bounds.Height));
        }
        catch (System.Exception e)
        {
            Console.WriteLine(e);
            Console.WriteLine("Source:\n{0}", Source?.Trim());
            throw;
        }
    }
    
    protected override Size MeasureOverride(Size availableSize) => new(Size, Size);

    protected override Size ArrangeOverride(Size finalSize) => new(Size, Size);

    // private void Regenerate(bool force = false)
    // {
    //     try
    //     {
    //         if (string.IsNullOrWhiteSpace(Source) || Bounds.Width <= 0 || Bounds.Height <= 0)
    //         {
    //             _image.Source = null;
    //             return;
    //         }
    //
    //
    //         // 把 DIP 转成像素尺寸（考虑缩放）
    //         var scale = VisualRoot?.RenderScaling ?? 1.0;
    //         var w = Math.Max(1, (int)Math.Round(Bounds.Width * scale));
    //         var h = Math.Max(1, (int)Math.Round(Bounds.Height * scale));
    //
    //         if (!force && w == _pxW && h == _pxH) return;
    //         _pxW = w; _pxH = h;
    //
    //         var hexColor = Foreground switch
    //         {
    //             SolidColorBrush brush => brush.Color.ToCssHex(),
    //             IImmutableSolidColorBrush immutableSolidColorBrush => immutableSolidColorBrush.Color.ToCssHex(),
    //             _ => null
    //         };
    //         Console.WriteLine("HexColor:  " + hexColor);
    //
    //         // 生成 RGBA/BGRA 数据
    //         var options = new SvgRenderOptions(
    //             Svg: Source, Width: Convert.ToUInt32(w), Height: Convert.ToUInt32(h), Color: hexColor);
    //         var bytes = options.Render();
    //     
    //
    //         if (bytes.Length < w * h * 4) return;
    //     
    //         var size = new PixelSize(w, h);
    //         var alpha = Premultiplied ? AlphaFormat.Premul : AlphaFormat.Unpremul;
    //         var fmt = SourceIsBgra ? PixelFormat.Bgra8888 : PixelFormat.Rgba8888;
    //     
    //         if (_wb == null || _wb.PixelSize != size || _wb.Format != fmt || _wb.AlphaFormat != alpha)
    //         {
    //             _wb?.Dispose();
    //             _wb = new WriteableBitmap(size, new Vector(96, 96), fmt, alpha);
    //         }
    //     
    //         using (var fb = _wb.Lock())
    //         {
    //             var srcStride = w * 4;
    //             for (var y = 0; y < h; y++)
    //             {
    //                 var dest = fb.Address + y * fb.RowBytes;
    //                 var srcOff = y * srcStride;
    //                 Marshal.Copy(bytes, srcOff, dest, srcStride);
    //             }
    //         }
    //         
    //     
    //         _image.Source = _wb;
    //     }
    //     catch (System.Exception e)
    //     {
    //         Console.WriteLine(e);
    //         Console.WriteLine("Source:\n{0}", Source?.Trim());
    //         throw;
    //     }
    // }
}