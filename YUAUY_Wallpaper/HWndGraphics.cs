using System.Drawing;

namespace YUAUY_Wallpaper;

public class HWndGraphics : IDisposable
{
    public HWndGraphics(IntPtr hWnd)
    {
        Handle = hWnd;
        Graphics = Graphics.FromHwnd(Handle);
        Rectangle = new Rectangle(0, 0, (int)Graphics.VisibleClipBounds.Width, (int)Graphics.VisibleClipBounds.Height);
    }
    public IntPtr Handle { get; private set; }
    public Graphics Graphics { get; private set; }
    public Rectangle Rectangle { get; private set; }

    public void Dispose()
    {
        Graphics.Dispose();
    }

    public Bitmap PrintToBitmap()
    {
        var bitmap = new Bitmap(Rectangle.Width, Rectangle.Height);
        using var graphics = Graphics.FromImage(bitmap);
        PrintToGraphics(graphics);
        return bitmap;
    }
    public void PrintToGraphics(Graphics graphics)
    {
        var hdc = graphics.GetHdc();
         _ = User32.PrintWindow(Handle, hdc, 0);
        graphics.ReleaseHdc(hdc);
    }
}