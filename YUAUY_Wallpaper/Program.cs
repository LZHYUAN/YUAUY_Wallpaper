using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using YUAUY_Wallpaper.Image;

namespace YUAUY_Wallpaper
{
    internal static class Program
    {
        static string ImagePath = @"D:\YUAN\Pictures\орем";
        static void Main()
        {
            var wallGraphics = new HWndGraphics(GetWorkerWHwnd());

            BufferedGraphicsManager.Current.MaximumBuffer = wallGraphics.Rectangle.Size;
            var wallBuffer = BufferedGraphicsManager.Current.Allocate(wallGraphics.Graphics, wallGraphics.Rectangle);
            wallGraphics.PrintWindowToGraphics(wallBuffer.Graphics);
            wallBuffer.Graphics.SmoothingMode = SmoothingMode.AntiAlias;

            ImagePicker imagePicker = new ImagePicker()
            {
                Sort = ImagePickerSort.Random_NotRepeating
            };
            imagePicker.ImageFolderList.Add(ImagePath);

            Random random = new Random((int)DateTime.Now.Ticks);
            var brush = new SolidBrush(Color.Transparent);

            for (int i = 1; i < 50; i++)
            {
                brush.Color = Color.FromArgb(i, 0, 0, 0);
                wallBuffer.Graphics.FillRectangle(brush, wallGraphics.Rectangle);
                wallBuffer.Render();
            }

            brush.Color = Color.FromArgb(5, 0, 0, 0);
            int mx = Screen.AllScreens.Min(_ => _.Bounds.X);
            int my = Screen.AllScreens.Min(_ => _.Bounds.Y);
            while (true)
            {
                // Task.Delay(1000).Wait();

                var img = imagePicker.Next();
                if (img == null) continue;
                var max = Math.Max(img.Width, img.Height);
                var rs = (double)random.Next(750, 1000) / max;
                var r = random.Next(-20, 20);
                if (rs > 1) rs = 1;
                var bmp = new Bitmap((int)(img.Width * rs) + 20, (int)(img.Height * rs) + 20, PixelFormat.Format32bppArgb);

                var g = Graphics.FromImage(bmp);
                g.Clear(Color.White);
                g.DrawImage(img, 10, 10, (int)(img.Width * rs), (int)(img.Height * rs));
                g.Dispose();

                int i = random.Next(Screen.AllScreens.Length);
                int x = random.Next(Screen.AllScreens[i].Bounds.Width) + Screen.AllScreens[i].Bounds.X;
                int y = random.Next(Screen.AllScreens[i].Bounds.Height) + Screen.AllScreens[i].Bounds.Y;

                wallBuffer.Graphics.FillRectangle(brush, wallGraphics.Rectangle);
                wallBuffer.Graphics.RotateTransform(r);
                wallBuffer.Graphics.DrawImage(bmp, x - mx - bmp.Width / 2, y - my - bmp.Height / 2);
                wallBuffer.Graphics.RotateTransform(-r);
                wallBuffer.Render();
                img.Dispose();
                bmp.Dispose();

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
    }
}