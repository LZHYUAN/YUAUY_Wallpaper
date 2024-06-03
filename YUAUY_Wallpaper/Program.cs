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
        wallBuffer.Graphics.SmoothingMode = SmoothingMode.HighSpeed;
        wallBuffer.Graphics.CompositingQuality = CompositingQuality.HighSpeed;
        wallBuffer.Graphics.PixelOffsetMode = PixelOffsetMode.HighSpeed;
        wallBuffer.Graphics.InterpolationMode = InterpolationMode.Low;


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

        var baseImage = wallGraphics.PrintToBitmap();
        var tmpg = Graphics.FromImage(baseImage);
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
                baseImage.Dispose();
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
            var nextBmp = new Bitmap((int)(img.Width * rs) + 20, (int)(img.Height * rs) + 20, PixelFormat.Format24bppRgb);

            var g = Graphics.FromImage(nextBmp);
            g.Clear(Color.White);
            g.DrawImage(img, 10, 10, (int)(img.Width * rs), (int)(img.Height * rs));
            g.Dispose();



            var s = Screen.AllScreens.Select(screen => (double)screen.Bounds.Width * screen.Bounds.Height).ToArray();
            s = s.Select(size => size / s.Sum()).ToArray();
            var st = random.NextDouble();
            int nextScreen = 0;
            foreach (var t in s)
            {
                st -= t;
                if (st <= 0) break;
                nextScreen++;
            }

            int x = random.Next(Screen.AllScreens[nextScreen].Bounds.Width - 400) + 200 + Screen.AllScreens[nextScreen].Bounds.X;
            int y = random.Next(Screen.AllScreens[nextScreen].Bounds.Height - 400) + 200 + Screen.AllScreens[nextScreen].Bounds.Y;



            int delay = 60;
            DateTime nextTime = DateTime.Now + TimeSpan.FromMilliseconds(delay);

            for (float time = 0f; time <= 1; time += first ? 0.25f : 0.03f)
            {
                float timeA= 1.2f / MathF.Cosh(time *3f* MathF.PI+0.65f); // 1 to 0

                wallBuffer.Graphics.DrawImageUnscaledAndClipped(baseImage, wallGraphics.Rectangle); //reset
                brush.Color = Color.FromArgb((int)(time * 10), 0, 0, 0);

                wallBuffer.Graphics.FillRectangle(brush, wallGraphics.Rectangle);

                wallBuffer.Graphics.TranslateTransform(x  + gx * timeA , y  + gy * timeA );
                wallBuffer.Graphics.RotateTransform(r *(1-timeA));
                wallBuffer.Graphics.DrawImageUnscaled(nextBmp, -nextBmp.Width / 2, -nextBmp.Height / 2);
                wallBuffer.Graphics.ResetTransform();
                wallBuffer.Render();

                var waitTime = nextTime - DateTime.Now;
                nextTime += TimeSpan.FromMilliseconds(delay);
                if (waitTime.Milliseconds > 0)
                    Task.Delay(waitTime).Wait();
            }

            tmpg.FillRectangle(brush, -1, -1, wallGraphics.Rectangle.Width + 2, wallGraphics.Rectangle.Height + 2);
            tmpg.TranslateTransform(x - nx, y - ny);
            tmpg.RotateTransform(r);
            tmpg.DrawImageUnscaled(nextBmp, -nextBmp.Width / 2, -nextBmp.Height / 2);
            tmpg.ResetTransform();

            wallBuffer.Graphics.DrawImageUnscaledAndClipped(baseImage, wallGraphics.Rectangle);
            wallBuffer.Render();

            //img.Dispose();
            //rbmp.Dispose();

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