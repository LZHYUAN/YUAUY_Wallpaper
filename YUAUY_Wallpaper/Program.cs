using System.Text;

namespace YUAUY_Wallpaper
{
    internal static class Program
    {
        static void Main()
        {
            IntPtr WorkerWHandel = IntPtr.Zero;
            User32.EnumWindows((hWnd, lParam) =>
            {
                var p = User32.FindWindowEx(hWnd, IntPtr.Zero, "SHELLDLL_DefView", null);
                if (p != IntPtr.Zero)
                {
                    WorkerWHandel = User32.FindWindowEx(IntPtr.Zero, hWnd, "WorkerW", null);
                    return false;
                }
                return true;
            }, IntPtr.Zero);
            var graphics = Graphics.FromHwnd(WorkerWHandel);
            graphics.Clear(Color.Green);

            //ApplicationConfiguration.Initialize();
            //Application.Run(new Form1());
        }
    }
}