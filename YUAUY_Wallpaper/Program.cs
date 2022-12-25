using System;
using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.Text;
using System.Windows.Forms;

namespace YUAUY_Wallpaper
{
    internal static class Program
    {
        static string ImagePath = @"D:\YUAN\Pictures\桌布";
        static void Main()
        {
            IntPtr WorkerWHwnd = GetWorkerWHwnd();
            BufferedGraphics buffer = CreateBufferedGraphicsFromHwnd(WorkerWHwnd, true);
            RectangleF rectangle = buffer.Graphics.VisibleClipBounds;

            buffer.Graphics.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;

            var images = new Queue<string>(Directory.GetFiles(ImagePath).Where(_ =>
            _.ToUpper().EndsWith(".JPEG") ||
            _.ToUpper().EndsWith(".JPG") ||
            _.ToUpper().EndsWith(".PNG")));

            rectangle.X = (Screen.AllScreens.Min(_ => _.Bounds.X));
            rectangle.Y = (Screen.AllScreens.Min(_ => _.Bounds.Y));
            Random random = new Random((int)DateTime.Now.Ticks);

            while (true)
            {
                var img = new Bitmap(images.Peek());
                var max = Math.Max(img.Width, img.Height);
                var rs = (double)random.Next(500, 1000) / max;
                if (rs > 1) rs = 1;
                var bmp = new Bitmap((int)(img.Width * rs) + 20, (int)(img.Height * rs) + 20, PixelFormat.Format32bppArgb);

                var g = Graphics.FromImage(bmp);
                g.Clear(Color.White);
                g.DrawImage(img, 10, 10, (int)(img.Width * rs), (int)(img.Height * rs));

                int s = random.Next(Screen.AllScreens.Length);
                int w = random.Next(Screen.AllScreens[s].Bounds.Width);
                int h = random.Next(Screen.AllScreens[s].Bounds.Height);
                int x = Screen.AllScreens[s].Bounds.X + w;
                int y = Screen.AllScreens[s].Bounds.Y + h;

                buffer.Graphics.FillRectangle(new SolidBrush(Color.FromArgb(5, 0, 0, 0)), 0, 0, rectangle.Width, rectangle.Height);
                var r = random.Next(-20, 20);
                buffer.Graphics.RotateTransform(r);
                buffer.Graphics.DrawImage(bmp, x - rectangle.X - bmp.Width / 2, y - rectangle.Y - bmp.Height / 2);
                buffer.Graphics.RotateTransform(-r);
                buffer.Render();

                images.Enqueue(images.Dequeue());

                Task.Delay(50).Wait();
            }

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