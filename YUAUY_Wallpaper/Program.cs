using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using YUAUY_Wallpaper.Image;

namespace YUAUY_Wallpaper;

internal static class Program
{
    static readonly string ImagePath = @"D:\YUAUY\Pictures\Dropbox\桌布";
    static void Main()
    {
    Restart:
        var wallGraphics = new HWndGraphics(GetWorkerWHwnd());
        BufferedGraphicsManager.Current.MaximumBuffer = wallGraphics.Rectangle.Size;
        var wallBuffer = BufferedGraphicsManager.Current.Allocate(wallGraphics.Graphics, wallGraphics.Rectangle);
        wallGraphics.PrintToGraphics(wallBuffer.Graphics);
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

        var tmpBmp = wallGraphics.PrintToBitmap();
        var tmpg = Graphics.FromImage(tmpBmp);
        tmpg.SmoothingMode = SmoothingMode.AntiAlias;

        brush.Color = Color.FromArgb(5, 0, 0, 0);
        int nx = Screen.AllScreens.Min(screen => screen.Bounds.Left);
        int ny = Screen.AllScreens.Min(screen => screen.Bounds.Top);
        int mx = Screen.AllScreens.Max(screen => screen.Bounds.Right);
        int my = Screen.AllScreens.Max(screen => screen.Bounds.Bottom);

        bool first = true;
        int firstCount = 0;
        while (true)
        {
            Task.Delay(first ? 0 : 8_000).Wait();
            // 土炮更新螢幕連接
            if (wallGraphics.Rectangle.Width != Screen.AllScreens.Max(screen => screen.Bounds.Right) || wallGraphics.Rectangle.Height != Screen.AllScreens.Max(screen => screen.Bounds.Bottom))
            {
                brush.Dispose();
                tmpg.Dispose();
                tmpBmp.Dispose();
                wallBuffer.Dispose();
                wallGraphics.Dispose();
                GC.Collect();
                goto Restart;
            }



            var img = imagePicker.Next();
            if (img == null) continue;
            var max = Math.Max(img.Width, img.Height);
            var rs = (double)random.Next(750, 1000) / max;
            var r = random.Next(-20, 20);
            var sr = (float)random.Next(50, 60) * (random.Next(2) * 2 - 1) + r;
            if (rs > 1) rs = 1;
            var rbmp = new Bitmap((int)(img.Width * rs) + 20, (int)(img.Height * rs) + 20, PixelFormat.Format32bppArgb);

            var g = Graphics.FromImage(rbmp);
            g.Clear(Color.White);
            g.DrawImage(img, 10, 10, (int)(img.Width * rs), (int)(img.Height * rs));
            g.Dispose();



            var s = Screen.AllScreens.Select(screen => (double)screen.Bounds.Width * screen.Bounds.Height).ToArray();
            s = s.Select(size => size / s.Sum()).ToArray();
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

            //wallBuffer.Graphics.FillRectangle(brush, -1, -1, wallGraphics.Rectangle.Width, wallGraphics.Rectangle.Height);


            var rrss = random.Next(4);
            float gx = rrss < 2 ? (rrss < 1 ? -1500 : mx + 1500) : random.Next(mx - nx);
            float gy = rrss > 1 ? (rrss > 2 ? -1500 : my + 1500) : random.Next(my - ny);

            for (float tt = 0; tt <= 5; tt += first ? 1 : 0.05f)
            {
                float t = (float)Math.Tanh(tt);
                float rt = (float)Math.Tanh((tt) / 1.1);
                //float rt = (float)Math.Sin(tt * Math.PI / 10);
                wallBuffer.Graphics.DrawImage(tmpBmp, 0, 0); //reset
                brush.Color = Color.FromArgb((int)(tt * 2), 0, 0, 0);
                wallBuffer.Graphics.FillRectangle(brush, wallGraphics.Rectangle);
                wallBuffer.Graphics.TranslateTransform((x - nx) - gx * t + gx, (y - ny) - gy * t + gy);
                wallBuffer.Graphics.RotateTransform(r - sr * rt + sr);
                wallBuffer.Graphics.DrawImage(rbmp, -rbmp.Width / 2, -rbmp.Height / 2);
                wallBuffer.Graphics.ResetTransform();
                wallBuffer.Render();
                Task.Delay(0).Wait();
            }

            tmpg.FillRectangle(brush, -1, -1, wallGraphics.Rectangle.Width + 2, wallGraphics.Rectangle.Height + 2);
            tmpg.TranslateTransform(x - nx, y - ny);
            tmpg.RotateTransform(r);
            tmpg.DrawImage(rbmp, -rbmp.Width / 2, -rbmp.Height / 2);
            tmpg.ResetTransform();

            img.Dispose();
            rbmp.Dispose();

            if (first && firstCount < Screen.AllScreens.Length * 5) firstCount++;
            else first = false;

        }

        // ReSharper disable once FunctionNeverReturns
    }
    static IntPtr GetWorkerWHwnd(bool callWorkerW = false)
    {
        // call progman create WorkerW
        if (callWorkerW)
        {
            var progman = User32.FindWindowEx(IntPtr.Zero, IntPtr.Zero, "Progman", null);
            User32.SendMessage(progman, 0x052C, new IntPtr(0xD), new IntPtr(0x1));
        }


        var workerWHwnd = IntPtr.Zero;
        User32.EnumWindows((hWnd, lParam) =>
        {
            IntPtr p = User32.FindWindowEx(hWnd, IntPtr.Zero, "SHELLDLL_DefView", null);
            if (p != IntPtr.Zero)
            {
                workerWHwnd = User32.FindWindowEx(IntPtr.Zero, hWnd, "WorkerW", null);
                return false; // Break Enum
            }
            return true;
        }, IntPtr.Zero);

        if (workerWHwnd != IntPtr.Zero)
            return workerWHwnd;

        if (callWorkerW)
            throw new Exception("Couldn't Find WorkerW");

        return GetWorkerWHwnd(true);
    }
}