using System.Diagnostics;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using YUAUY_Wallpaper.Image;
using static System.Windows.Forms.VisualStyles.VisualStyleElement.TaskbarClock;

namespace YUAUY_Wallpaper;

internal static class Program
{
    static readonly string ImagePath = @"D:\YUAUY\Pictures\Dropbox\桌布";
    //static readonly string ImagePath = @"D:\YUAUY\Desktop\CC";
    static void Main()
    {

 Application.Run(new Form1());
        


    //SetParent(myWindowHwnd, workerw);

    //        var wwh = GetWorkerWHwnd(out var ssh);

    //        User32.GetWindowRect(wwh, out var wwrect);

    //        var hInstance = User32.GetModuleHandle(null);
    //        var hChild = User32.CreateWindowEx(
    //    0,
    //    "STATIC",               // 使用系統內建 Static control
    //    "This is child window",
    //    User32.WS_CHILD | User32.WS_VISIBLE,
    // //User32.WS_VISIBLE,
    //    wwrect.Left,
    //    wwrect.Top,
    //    wwrect.Right - wwrect.Left,
    //    wwrect.Bottom - wwrect.Top ,
    //    //0,                  // parent window handle
    //    wwh,                  // parent window handle
    //    IntPtr.Zero,
    //    hInstance,
    //    IntPtr.Zero
    //);
    //        User32.SetWindowPos(
    //    1,
    //    0,
    //    0, 0, 0, 0,
    //    User32.SWP_NOMOVE | User32.SWP_NOSIZE | User32.SWP_NOACTIVATE
    //);
    //        User32.ShowWindow(hChild, 5);
    //        User32.UpdateWindow(hChild);

    Restart:
        //var wallGraphics = new HWndGraphics(GetWorkerWHwnd(out var _));
        var wallGraphics = new HWndGraphics(00);
        var wallBuffer = BufferedGraphicsManager.Current.Allocate(wallGraphics.Graphics, wallGraphics.Rectangle);
        BufferedGraphicsManager.Current.MaximumBuffer = wallGraphics.Rectangle.Size;
        wallBuffer.Graphics.SmoothingMode = SmoothingMode.HighSpeed;
        wallBuffer.Graphics.CompositingQuality = CompositingQuality.HighSpeed;
        wallBuffer.Graphics.PixelOffsetMode = PixelOffsetMode.HighSpeed;
        wallBuffer.Graphics.InterpolationMode = InterpolationMode.Low;
        wallGraphics.PrintToGraphics(wallBuffer.Graphics);


        ImagePicker imagePicker = new ImagePicker()
        {
            Sort = ImagePickerSort.Random_NotRepeating
        };
        imagePicker.ImageFolderList.Add(ImagePath);

        Random random = new Random((int)DateTime.Now.Ticks);
        var brushA = new SolidBrush(Color.Transparent);
        var brushB = new SolidBrush(Color.Transparent);

        for (int i = 1; i < 50; i++)
        {
            brushA.Color = Color.FromArgb(i, 0, 0, 0);
            wallBuffer.Graphics.FillRectangle(brushA, wallGraphics.Rectangle);
            wallBuffer.Render();
        }

        var baseImage = wallGraphics.PrintToBitmap();
        //baseImage = baseImage.Clone(wallGraphics.Rectangle, PixelFormat.Format32bppArgb);
        var baseImageGraphics = Graphics.FromImage(baseImage);
        List<Bitmap?> lastBmp = Screen.AllScreens.Select<Screen, Bitmap?>(_ => null).ToList();
        baseImageGraphics.SmoothingMode = SmoothingMode.AntiAlias;

        brushA.Color = Color.FromArgb(5, 0, 0, 0);
        int nx = Screen.AllScreens.Min(screen => screen.Bounds.Left);
        int ny = Screen.AllScreens.Min(screen => screen.Bounds.Top);
        int mx = Screen.AllScreens.Max(screen => screen.Bounds.Right);
        int my = Screen.AllScreens.Max(screen => screen.Bounds.Bottom);

        int[] cx = Screen.AllScreens.Select(s => s.Bounds.Left + s.Bounds.Width / 2).ToArray();
        int[] cy = Screen.AllScreens.Select(s => s.Bounds.Top + s.Bounds.Height / 2).ToArray();

        bool first = true;
        int firstCount = 0;
        while (true)
        {
#if DEBUG
            Task.Delay(first ? 0 : 1_000).Wait();
#else
            Task.Delay(first ? 0 : 30_000).Wait();
#endif 

            // 土炮更新螢幕連接
            //if (wallGraphics.Rectangle.Width != Screen.AllScreens.Max(screen => screen.Bounds.Right) || wallGraphics.Rectangle.Height != Screen.AllScreens.Max(screen => screen.Bounds.Bottom))
            //{
            //    brushA.Dispose();
            //    brushB.Dispose();
            //    baseImageGraphics.Dispose();
            //    baseImage.Dispose();
            //    wallBuffer.Dispose();
            //    wallGraphics.Dispose();
            //    GC.Collect();
            //    goto Restart;
            //}


            // next screen
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


            // next img
            var img = imagePicker.Next();
            if (img == null) continue;
            var irs = (float)img.Width / img.Height;
            var srs = (float)Screen.AllScreens[nextScreen].Bounds.Width / Screen.AllScreens[nextScreen].Bounds.Height;
            var rsx = Math.Abs(irs - srs) * 0.1 + 0.8;
            Debug.WriteLine(Math.Abs(irs - srs));
            Bitmap nextBmp;
            if (irs > srs) // 比螢幕寬
                nextBmp = new Bitmap((int)(Screen.AllScreens[nextScreen].Bounds.Width * rsx) + 20, (int)(Screen.AllScreens[nextScreen].Bounds.Width * rsx / img.Width * img.Height) + 20, PixelFormat.Format24bppRgb);
            else // 比螢幕高
                nextBmp = new Bitmap((int)(Screen.AllScreens[nextScreen].Bounds.Height * rsx / img.Height * img.Width) + 20, (int)(Screen.AllScreens[nextScreen].Bounds.Height * rsx) + 20, PixelFormat.Format24bppRgb);

            var g = Graphics.FromImage(nextBmp);
            g.Clear(Color.White);
            g.DrawImage(img, 10, 10, nextBmp.Width - 20, nextBmp.Height - 20);
            g.Dispose();
            lastBmp[nextScreen] ??= nextBmp;


            // next loc
            float x = random.Next((int)(Screen.AllScreens[nextScreen].Bounds.Width * 0.4));
            if (x < Screen.AllScreens[nextScreen].Bounds.Width * 0.2) x -= Screen.AllScreens[nextScreen].Bounds.Width * 0.4f;

            float y = random.Next((int)(Screen.AllScreens[nextScreen].Bounds.Height * 0.4));
            if (y < Screen.AllScreens[nextScreen].Bounds.Height * 0.2) y -= Screen.AllScreens[nextScreen].Bounds.Height * 0.4f;
            //if (y < 0) y += Screen.AllScreens[nextScreen].Bounds.Height;

            var r = random.Next(-20, 20);
            //var rrss = random.NextSingle()*0.2f+0.7f;
            var rrss = 0.2f;

            int delay = 40;
            DateTime nextTime = DateTime.Now + TimeSpan.FromMilliseconds(delay);
            for (float time = 0f; time <= 1; time += first ? 1 : 0.03f)
            {
                float timeA = 1 - 1.2f / MathF.Cosh(time * 3f * MathF.PI + 0.65f); // 0 to 1

                wallBuffer.Graphics.DrawImageUnscaledAndClipped(baseImage, wallGraphics.Rectangle); //reset
                //brushB.Color = Color.FromArgb((int)(time * 4), 0, 0, 0);
                //baseImageGraphics.FillRectangle(brushB, Screen.AllScreens[nextScreen].Bounds);


                wallBuffer.Graphics.TranslateTransform(cx[nextScreen] + x * timeA, cy[nextScreen] + y * timeA);
                wallBuffer.Graphics.RotateTransform(r * timeA);

                var rrsst = 1 - rrss * timeA;
                brushA.Color = Color.FromArgb((int)(timeA * 175), 0, 0, 0);
                wallBuffer.Graphics.DrawImage(lastBmp[nextScreen]!, -lastBmp[nextScreen]!.Width * rrsst / 2, -lastBmp[nextScreen]!.Height * rrsst / 2, lastBmp[nextScreen]!.Width * rrsst, lastBmp[nextScreen]!.Height * rrsst);
                wallBuffer.Graphics.FillRectangle(brushA, -lastBmp[nextScreen]!.Width * rrsst / 2, -lastBmp[nextScreen]!.Height * rrsst / 2, lastBmp[nextScreen]!.Width * rrsst, lastBmp[nextScreen]!.Height * rrsst);

                wallBuffer.Graphics.ResetTransform();

                if (time >= 0.75)
                {
                    wallBuffer.Graphics.DrawImageUnscaled(nextBmp, cx[nextScreen] - nextBmp.Width / 2, cy[nextScreen] - nextBmp.Height / 2);
                }
                else if (time >= 0.5)
                {
                    float timeB = time * 4 - 2;
                    timeB = 1 - 1 / MathF.Cosh(timeB * 2f * MathF.PI); // 0 to 1
                    wallBuffer.Graphics.DrawImageUnscaled(nextBmp, cx[nextScreen] - nextBmp.Width / 2, cy[nextScreen] - nextBmp.Height / 2);
                    wallBuffer.Graphics.FillRectangle(Brushes.White, cx[nextScreen] - nextBmp.Width / 2f, cy[nextScreen] + nextBmp.Height * (timeB - 0.5f), nextBmp.Width, nextBmp.Height * (1 - timeB));
                }
                else if (time >= 0.25)
                {
                    float timeB = time * 4 - 1;
                    timeB = 1 / MathF.Cosh((timeB - 1) * 2f * MathF.PI); // 0 to 1
                    wallBuffer.Graphics.FillRectangle(Brushes.White, cx[nextScreen] - nextBmp.Width / 2f, cy[nextScreen] - nextBmp.Height / 2f, nextBmp.Width, nextBmp.Height * timeB);
                }
                for (int i = 0; i < Screen.AllScreens.Length; i++)
                {
                    if (i == nextScreen) continue;
                    lastBmp[i] ??= nextBmp;
                    wallBuffer.Graphics.DrawImageUnscaled(lastBmp[i]!, cx[i] - lastBmp[i]!.Width / 2, cy[i] - lastBmp[i]!.Height / 2);
                }


                wallBuffer.Render();

                var waitTime = nextTime - DateTime.Now;
                nextTime += TimeSpan.FromMilliseconds(delay);
                if (waitTime.Milliseconds > 0)
                    Task.Delay(waitTime).Wait();
            }

            //baseImageGraphics.FillRectangle(brushB, Screen.AllScreens[nextScreen].Bounds);
            baseImageGraphics.TranslateTransform(cx[nextScreen] + x, cy[nextScreen] + y);
            baseImageGraphics.RotateTransform(r);
            Graphics.FromImage(lastBmp[nextScreen]!).FillRectangle(brushA, 0, 0, lastBmp[nextScreen]!.Width, lastBmp[nextScreen]!.Height);
            baseImageGraphics.DrawImage(lastBmp[nextScreen]!, -lastBmp[nextScreen]!.Width * (1 - rrss) / 2, -lastBmp[nextScreen]!.Height * (1 - rrss) / 2, lastBmp[nextScreen]!.Width * (1 - rrss), lastBmp[nextScreen]!.Height * (1 - rrss));
            baseImageGraphics.ResetTransform();

            lastBmp[nextScreen] = nextBmp;
            wallBuffer.Graphics.DrawImageUnscaledAndClipped(baseImage, wallGraphics.Rectangle);
            for (int i = 0; i < Screen.AllScreens.Length; i++)
            {
                wallBuffer.Graphics.DrawImageUnscaled(lastBmp[i]!, cx[i] - lastBmp[i]!.Width / 2, cy[i] - lastBmp[i]!.Height / 2);
            }
            wallBuffer.Render();

            //img.Dispose();
            //rbmp.Dispose();

            if (first && firstCount < Screen.AllScreens.Length * 10) firstCount++;
            else first = false;

        }

        // ReSharper disable once FunctionNeverReturns
    }
    static IntPtr GetWorkerWHwnd(out nint shelldll, bool callWorkerW = false)
    {
        // call progman create WorkerW
        var progman = User32.FindWindowEx(IntPtr.Zero, IntPtr.Zero, "Progman", null);
        if (callWorkerW)
        {
            User32.SendMessage(progman, 0x052C, new IntPtr(0xD), new IntPtr(0x1));
        }


        var workerWHwnd = IntPtr.Zero;
        // Windows >= 24H2
        workerWHwnd = User32.FindWindowEx(progman, IntPtr.Zero, "WorkerW", null);
        shelldll = User32.FindWindowEx(progman, IntPtr.Zero, "SHELLDLL_DefView", null);
        // Windows < 24H2
        //User32.EnumWindows((hWnd, lParam) =>
        //{
        //    IntPtr p = User32.FindWindowEx(hWnd, IntPtr.Zero, "SHELLDLL_DefView", null);
        //    if (p != IntPtr.Zero)
        //    {
        //        workerWHwnd = User32.FindWindowEx(IntPtr.Zero, hWnd, "WorkerW", null);
        //        return false; // Break Enum
        //    }
        //    return true;
        //}, IntPtr.Zero);

        if (workerWHwnd != IntPtr.Zero)
            return workerWHwnd;

        if (callWorkerW)
            throw new Exception("Couldn't Find WorkerW");

        return GetWorkerWHwnd(out var _, true);
    }
}