using System;
using System.Drawing;
using System.Text;
using System.Windows.Forms;

namespace YUAUY_Wallpaper
{
    internal static class Program
    {
        static void Main()
        {
            IntPtr WorkerWHwnd = GetWorkerWHwnd();
            BufferedGraphics buffer = CreateBufferedGraphicsFromHwnd(WorkerWHwnd);
            RectangleF rectangle = buffer.Graphics.VisibleClipBounds;
            var brush = new SolidBrush(Color.Transparent);

            buffer.Graphics.FillRectangle(Brushes.Blue, -100, -100, 200, 200);
            for (int i = 0; i < 255; i++)
            {

                brush.Color = Color.FromArgb(i, 0, 0, 0);
                buffer.Graphics.FillRectangle(brush, rectangle);
                buffer.Render();
                Task.Delay(0).Wait();
            }
            //ApplicationConfiguration.Initialize();
            //Application.Run(new Form1());
        }
        static IntPtr GetWorkerWHwnd()
        {
            IntPtr WorkerWHwnd = IntPtr.Zero;
            User32.EnumWindows((hWnd, lParam) =>
            {
                IntPtr p = User32.FindWindowEx(hWnd, IntPtr.Zero, "SHELLDLL_DefView", null);
                if (p != IntPtr.Zero)
                {
                    WorkerWHwnd = User32.FindWindowEx(IntPtr.Zero, hWnd, "WorkerW", null);
                    return false; // Break Enum
                }
                return true;
            }, IntPtr.Zero);

            return WorkerWHwnd != IntPtr.Zero ? WorkerWHwnd : throw new Exception("Couldn't Find WorkerW");
        }
        static BufferedGraphics CreateBufferedGraphicsFromHwnd(IntPtr Hwnd, bool printWindow = false)
        {
            var graphics = System.Drawing.Graphics.FromHwnd(Hwnd);

            Rectangle rectangle = new Rectangle(0, 0, (int)graphics.VisibleClipBounds.Width, (int)graphics.VisibleClipBounds.Height);

            BufferedGraphics buffer = BufferedGraphicsManager.Current.Allocate(graphics, rectangle);

            if (printWindow) // 繪製原有的畫面在buffer上
            {
                var bufferHdc = buffer.Graphics.GetHdc();
                bool success = User32.PrintWindow(Hwnd, bufferHdc, 0);
                buffer.Graphics.ReleaseHdc(bufferHdc);
            }

            return buffer;
        }

    }
}