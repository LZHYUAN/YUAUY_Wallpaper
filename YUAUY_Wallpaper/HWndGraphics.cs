using System;
using System.Collections.Generic;
using System.Drawing;
using System.Linq;
using System.Reflection.Metadata;
using System.Text;
using System.Threading.Tasks;

namespace YUAUY_Wallpaper
{
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

        public void CopyScreenTo(Bitmap bitmap)
        {
            using (Graphics graphics = Graphics.FromImage(bitmap))
            {
                CopyScreenTo(graphics);
            }
        }
        public void CopyScreenTo(Graphics graphics)
        {
            var Hdc = graphics.GetHdc();
            bool success = User32.PrintWindow(Handle, Hdc, 0);
            graphics.ReleaseHdc(Hdc);
        }
    }
}
