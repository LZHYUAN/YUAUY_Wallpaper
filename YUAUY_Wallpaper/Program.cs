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

            var tmpBmp = wallGraphics.PrintWindowToBitmap();
            var tmpg = Graphics.FromImage(tmpBmp);
            tmpg.SmoothingMode = SmoothingMode.AntiAlias;

            brush.Color = Color.FromArgb(5, 0, 0, 0);
            int nx = Screen.AllScreens.Min(_ => _.Bounds.Left);
            int ny = Screen.AllScreens.Min(_ => _.Bounds.Top);
            int mx = Screen.AllScreens.Max(_ => _.Bounds.Right);
            int my = Screen.AllScreens.Max(_ => _.Bounds.Bottom);
            while (true)
            {
                Task.Delay(1000).Wait();

                var img = imagePicker.Next();
                if (img == null) continue;
                var max = Math.Max(img.Width, img.Height);
                var rs = (double)random.Next(750, 1000) / max;
                var r = random.Next(-20, 20);
                var sr = (float)random.Next(50, 60)*(random.Next(2)*2-1) + r;
                if (rs > 1) rs = 1;
                var rbmp = new Bitmap((int)(img.Width * rs) + 20, (int)(img.Height * rs) + 20, PixelFormat.Format32bppArgb);

                var g = Graphics.FromImage(rbmp);
                g.Clear(Color.White);
                g.DrawImage(img, 10, 10, (int)(img.Width * rs), (int)(img.Height * rs));
                g.Dispose();



                var s = Screen.AllScreens.Select(_ => (double)_.Bounds.Width * _.Bounds.Height).ToArray();
                s = s.Select(_ => _ / s.Sum()).ToArray();
                var st = random.NextDouble();
                int i = 0;
                foreach (var t in s)
                {
                    st -= t;
                    if (st <= 0) break;
                    i++;
                }
                int x = random.Next(Screen.AllScreens[i].Bounds.Width - 400) + 200 + Screen.AllScreens[i].Bounds.X;
                int y = random.Next(Screen.AllScreens[i].Bounds.Height - 400) + 200 + Screen.AllScreens[i].Bounds.Y;

                wallBuffer.Graphics.FillRectangle(brush, wallGraphics.Rectangle);


                var rrss = random.Next(4);
                float gx = rrss < 2 ? (rrss < 1 ? -1500 : mx + 1500) : random.Next(mx - nx);
                float gy = rrss > 1 ? (rrss > 2 ? -1500 : my + 1500) : random.Next(my - ny);

                for (float tt = 0; tt <= 5; tt += 0.05f)
                {
                    float t =(float)Math.Tanh(tt);
                    float rt =(float)Math.Tanh((tt)/1.1);
                    //float rt = (float)Math.Sin(tt * Math.PI / 10);
                    wallBuffer.Graphics.DrawImage(tmpBmp, 0, 0);
                    brush.Color = Color.FromArgb((int)(tt*2), 0, 0, 0);
                    wallBuffer.Graphics.FillRectangle(brush, wallGraphics.Rectangle);
                    wallBuffer.Graphics.TranslateTransform((x - nx) - gx * t + gx, (y - ny) - gx * t + gx);
                    wallBuffer.Graphics.RotateTransform(r - sr * rt + sr);
                    wallBuffer.Graphics.DrawImage(rbmp, -rbmp.Width / 2, -rbmp.Height / 2);
                    wallBuffer.Graphics.ResetTransform();
                    wallBuffer.Render();
                }

                tmpg.FillRectangle(brush, wallGraphics.Rectangle);
                tmpg.TranslateTransform(x - nx, y - ny);
                tmpg.RotateTransform(r);
                tmpg.DrawImage(rbmp, -rbmp.Width / 2, -rbmp.Height / 2);
                tmpg.ResetTransform();

                img.Dispose();
                rbmp.Dispose();

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