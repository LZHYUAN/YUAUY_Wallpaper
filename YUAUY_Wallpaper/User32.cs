using System;
using System.Collections;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.InteropServices;
using System.Text;
using System.Threading.Tasks;

namespace YUAUY_Wallpaper
{
    public static class User32
    {
        [DllImport("user32.dll")]
        public static extern IntPtr GetDCEx(IntPtr hWnd, IntPtr hrgnClip, DeviceContextValues flags);
        public enum DeviceContextValues : uint
        {
            /// <summary>DCX_WINDOW: Returns a DC that corresponds to the window rectangle rather than the client rectangle.<para/>
            /// DCX_WINDOW：返回對應於窗口矩形而不是客戶端矩形的 DC。</summary>
            Window = 0x00000001,
            /// <summary>DCX_CACHE: Returns a DC from the cache, rather than the OWNDC or CLASSDC window. Essentially overrides CS_OWNDC and CS_CLASSDC.<para/>
            /// DCX_CACHE：從緩存中返回一個 DC，而不是 OWNDC 或 CLASSDC 窗口。 實質上覆蓋 CS_OWNDC 和 CS_CLASSDC</summary>
            Cache = 0x00000002,
            /// <summary>DCX_NORESETATTRS: Does not reset the attributes of this DC to the default attributes when this DC is released.<para/>
            /// DCX_NORESETATTRS：釋放此DC時不將此DC的屬性重置為默認屬性</summary>
            NoResetAttrs = 0x00000004,
            /// <summary>DCX_CLIPCHILDREN: Excludes the visible regions of all child windows below the window identified by hWnd.<para/>
            /// DCX_CLIPCHILDREN：排除hWnd標識的窗口下方所有子窗口的可見區域</summary>
            ClipChildren = 0x00000008,
            /// <summary>DCX_CLIPSIBLINGS: Excludes the visible regions of all sibling windows above the window identified by hWnd.<para/>
            /// DCX_CLIPSIBLINGS：排除hWnd標識的窗口上方所有同級窗口的可見區域</summary>
            ClipSiblings = 0x00000010,
            /// <summary>DCX_PARENTCLIP: Uses the visible region of the parent window. The parent's WS_CLIPCHILDREN and CS_PARENTDC style bits are ignored. The origin is set to the upper-left corner of the window identified by hWnd.<para/>
            /// DCX_PARENTCLIP：使用父窗口的可見區域。 忽略父項的 WS_CLIPCHILDREN 和 CS_PARENTDC 樣式位。 原點設置為 hWnd 標識的窗口的左上角</summary>
            ParentClip = 0x00000020,
            /// <summary>DCX_EXCLUDERGN: The clipping region identified by hrgnClip is excluded from the visible region of the returned DC.<para/>
            /// DCX_EXCLUDERGN：將hrgnClip標識的裁剪區域排除在返回DC的可見區域之外</summary>
            ExcludeRgn = 0x00000040,
            /// <summary>DCX_INTERSECTRGN: The clipping region identified by hrgnClip is intersected with the visible region of the returned DC.<para/>
            /// DCX_INTERSECTRGN：hrgnClip標識的裁剪區域與返回DC的可見區域相交</summary>
            IntersectRgn = 0x00000080,
            /// <summary>DCX_EXCLUDEUPDATE: Unknown...Undocumented<para/>
            /// DCX_EXCLUDEUPDATE：未知...未記錄</summary>
            ExcludeUpdate = 0x00000100,
            /// <summary>DCX_INTERSECTUPDATE: Unknown...Undocumented<para/>
            /// DCX_INTERSECTUPDATE：未知...未記錄</summary>
            IntersectUpdate = 0x00000200,
            /// <summary>DCX_LOCKWINDOWUPDATE: Allows drawing even if there is a LockWindowUpdate call in effect that would otherwise exclude this window. Used for drawing during tracking.<para/>
            /// DCX_LOCKWINDOWUPDATE：允許繪圖，即使有一個有效的 LockWindowUpdate 調用，否則會排除此窗口。 跟踪時用於繪製</summary>
            LockWindowUpdate = 0x00000400,
            /// <summary>DCX_VALIDATE When specified with DCX_INTERSECTUPDATE, causes the DC to be completely validated. Using this function with both DCX_INTERSECTUPDATE and DCX_VALIDATE is identical to using the BeginPaint function.<para/>
            /// DCX_VALIDATE 當與 DCX_INTERSECTUPDATE 一起指定時，導致 DC 被完全驗證。 將此函數與 DCX_INTERSECTUPDATE 和 DCX_VALIDATE 一起使用與使用 BeginPaint 函數相同</summary>
            Validate = 0x00200000,
        }

        [DllImport("user32.dll")]
        public static extern IntPtr FindWindow(string lpClassName, string lpWindowName);

        [DllImport("user32.dll")]
        public static extern IntPtr SetParent(IntPtr hWndChild, IntPtr hWndNewParent);

        [DllImport("user32.dll")]
        [return: MarshalAs(UnmanagedType.Bool)]
        public static extern bool EnumWindows(EnumWindowsProc lpEnumFunc, IntPtr lParam);
        public delegate bool EnumWindowsProc(IntPtr hWnd, IntPtr lParam);

        [DllImport("user32.dll", SetLastError = true)]
        public static extern IntPtr FindWindowEx(IntPtr parentHandle, IntPtr hWndChildAfter, string className, string windowTitle);

    }
}
