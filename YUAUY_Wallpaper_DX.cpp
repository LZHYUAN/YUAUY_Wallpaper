#include "YUAUY_Wallpaper_DX.h"

#include <algorithm>
#include <cmath>

namespace
{
constexpr wchar_t kWindowClassName[] = L"YUAUYWallpaperWindow";
constexpr wchar_t kWindowTitle[] = L"YUAUY Wallpaper DX";
constexpr UINT_PTR kAnimationTimerId = 1;
constexpr UINT_PTR kHostValidationTimerId = 2;
constexpr UINT kFrameIntervalMs = 16;
constexpr UINT kHostValidationIntervalMs = 2000;
constexpr double kRotationStepRadians = 0.04;
constexpr UINT kSpawnWorkerMessage = 0x052C;

struct DesktopHostContext
{
    HWND progman = nullptr;
    HWND shellView = nullptr;
    HWND compatibilityWorkerW = nullptr;
    bool shellViewUnderProgman = false;
    RECT virtualScreen = {};
};

struct AppState
{
    double angleRadians = 0.0;
    bool embedded = false;
    DesktopHostContext host = {};
};

struct WorkerWSearch
{
    HWND shellViewHost = nullptr;
    HWND shellView = nullptr;
    HWND workerW = nullptr;
};

BOOL CALLBACK FindWorkerWCallback(HWND hwnd, LPARAM lParam)
{
    auto* search = reinterpret_cast<WorkerWSearch*>(lParam);
    if (search == nullptr)
    {
        return FALSE;
    }

    HWND shellView = FindWindowExW(hwnd, nullptr, L"SHELLDLL_DefView", nullptr);
    if (shellView == nullptr)
    {
        return TRUE;
    }

    search->shellViewHost = hwnd;
    search->shellView = shellView;
    search->workerW = FindWindowExW(nullptr, hwnd, L"WorkerW", nullptr);
    return FALSE;
}

RECT GetPrimaryScreenRect()
{
    RECT rect = {};
    rect.left = 0;
    rect.top = 0;
    rect.right = GetSystemMetrics(SM_CXSCREEN);
    rect.bottom = GetSystemMetrics(SM_CYSCREEN);
    return rect;
}

RECT GetVirtualScreenRect()
{
    RECT rect = {};
    rect.left = GetSystemMetrics(SM_XVIRTUALSCREEN);
    rect.top = GetSystemMetrics(SM_YVIRTUALSCREEN);
    rect.right = rect.left + GetSystemMetrics(SM_CXVIRTUALSCREEN);
    rect.bottom = rect.top + GetSystemMetrics(SM_CYVIRTUALSCREEN);
    return rect;
}

DesktopHostContext ResolveDesktopHostContext()
{
    DesktopHostContext context = {};
    context.virtualScreen = GetVirtualScreenRect();
    context.progman = FindWindowW(L"Progman", nullptr);
    if (context.progman == nullptr)
    {
        return context;
    }

    DWORD_PTR result = 0;
    SendMessageTimeoutW(
        context.progman,
        kSpawnWorkerMessage,
        0,
        0,
        SMTO_NORMAL,
        1000,
        &result);

    context.shellView = FindWindowExW(context.progman, nullptr, L"SHELLDLL_DefView", nullptr);
    context.shellViewUnderProgman = (context.shellView != nullptr);

    WorkerWSearch search = {};
    EnumWindows(FindWorkerWCallback, reinterpret_cast<LPARAM>(&search));
    if (context.shellView == nullptr)
    {
        context.shellView = search.shellView;
    }

    context.compatibilityWorkerW = search.workerW;
    return context;
}

bool IsHostRelationshipValid(const AppState& state, HWND window)
{
    if (!IsWindow(window))
    {
        return false;
    }

    HWND parent = GetParent(window);
    if (!IsWindow(parent))
    {
        return false;
    }

    if (state.host.progman != nullptr && parent == state.host.progman)
    {
        return true;
    }

    return state.host.compatibilityWorkerW != nullptr && parent == state.host.compatibilityWorkerW;
}

void ApplyLayeredWindowSettings(HWND window)
{
    SetLayeredWindowAttributes(window, 0, 0xFF, LWA_ALPHA);
}

void NormalizeAsWallpaperChild(HWND window)
{
    LONG_PTR style = GetWindowLongPtrW(window, GWL_STYLE);
    style |= WS_CHILD | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS;
    style &= ~static_cast<LONG_PTR>(WS_POPUP | WS_OVERLAPPEDWINDOW | WS_CAPTION | WS_THICKFRAME | WS_MINIMIZEBOX | WS_MAXIMIZEBOX | WS_SYSMENU);
    SetWindowLongPtrW(window, GWL_STYLE, style);

    LONG_PTR exStyle = GetWindowLongPtrW(window, GWL_EXSTYLE);
    exStyle |= WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED;
    exStyle &= ~static_cast<LONG_PTR>(WS_EX_APPWINDOW);
    SetWindowLongPtrW(window, GWL_EXSTYLE, exStyle);
    ApplyLayeredWindowSettings(window);
}

void NormalizeAsPreviewWindow(HWND window)
{
    LONG_PTR style = GetWindowLongPtrW(window, GWL_STYLE);
    style &= ~static_cast<LONG_PTR>(WS_CHILD);
    style |= WS_OVERLAPPEDWINDOW | WS_VISIBLE;
    SetWindowLongPtrW(window, GWL_STYLE, style);

    LONG_PTR exStyle = GetWindowLongPtrW(window, GWL_EXSTYLE);
    exStyle &= ~static_cast<LONG_PTR>(WS_EX_NOACTIVATE);
    SetWindowLongPtrW(window, GWL_EXSTYLE, exStyle);
    ApplyLayeredWindowSettings(window);
}

RECT GetParentRelativeRect(HWND parent, const RECT& screenRect)
{
    POINT topLeft = { screenRect.left, screenRect.top };
    POINT bottomRight = { screenRect.right, screenRect.bottom };

    ScreenToClient(parent, &topLeft);
    ScreenToClient(parent, &bottomRight);

    RECT rect = {};
    rect.left = topLeft.x;
    rect.top = topLeft.y;
    rect.right = bottomRight.x;
    rect.bottom = bottomRight.y;
    return rect;
}

bool PositionWallpaperChild(HWND window, HWND parent, const DesktopHostContext& host)
{
    RECT targetRect = GetParentRelativeRect(parent, host.virtualScreen);
    const int width = targetRect.right - targetRect.left;
    const int height = targetRect.bottom - targetRect.top;
    if (width <= 0 || height <= 0)
    {
        return false;
    }

    HWND insertAfter = HWND_BOTTOM;
    if (parent == host.progman && host.shellViewUnderProgman && IsWindow(host.shellView) && GetParent(host.shellView) == parent)
    {
        insertAfter = host.shellView;
    }

    if (!SetWindowPos(
            window,
            insertAfter,
            targetRect.left,
            targetRect.top,
            width,
            height,
            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED))
    {
        return false;
    }

    return RedrawWindow(window, nullptr, nullptr, RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN) != FALSE;
}

bool EmbedIntoHost(HWND window, HWND parent, const DesktopHostContext& host)
{
    if (!IsWindow(parent))
    {
        return false;
    }

    NormalizeAsWallpaperChild(window);
    SetLastError(0);
    if (SetParent(window, parent) == nullptr && GetLastError() != 0)
    {
        return false;
    }

    return PositionWallpaperChild(window, parent, host);
}

bool ShowPreviewWindow(HWND window)
{
    NormalizeAsPreviewWindow(window);
    SetLastError(0);
    SetParent(window, nullptr);

    RECT virtualScreen = GetVirtualScreenRect();
    const int width = std::max<int>(960, (virtualScreen.right - virtualScreen.left) / 2);
    const int height = std::max<int>(540, (virtualScreen.bottom - virtualScreen.top) / 2);
    const int x = virtualScreen.left + 80;
    const int y = virtualScreen.top + 80;

    return SetWindowPos(
               window,
               HWND_TOP,
               x,
               y,
               width,
               height,
               SWP_SHOWWINDOW | SWP_FRAMECHANGED) != FALSE;
}

bool AttachToDesktopHost(HWND window, AppState& state)
{
    state.host = ResolveDesktopHostContext();

    if (IsWindow(state.host.progman) && EmbedIntoHost(window, state.host.progman, state.host))
    {
        state.embedded = true;
        return true;
    }

    if (IsWindow(state.host.compatibilityWorkerW) && EmbedIntoHost(window, state.host.compatibilityWorkerW, state.host))
    {
        state.embedded = true;
        return true;
    }

    state.embedded = false;
    return ShowPreviewWindow(window);
}

void DrawRotatingSquare(HDC hdc, const RECT& clientRect, double angleRadians, bool embedded)
{
    const int width = clientRect.right - clientRect.left;
    const int height = clientRect.bottom - clientRect.top;
    const int centerX = width / 2;
    const int centerY = height / 2;
    const int halfSize = std::min(width, height) / 5;

    POINT square[4] = {
        { -halfSize, -halfSize },
        { halfSize, -halfSize },
        { halfSize, halfSize },
        { -halfSize, halfSize },
    };

    POINT rotated[4] = {};
    const double cosAngle = std::cos(angleRadians);
    const double sinAngle = std::sin(angleRadians);

    for (int i = 0; i < 4; ++i)
    {
        const double rotatedX = square[i].x * cosAngle - square[i].y * sinAngle;
        const double rotatedY = square[i].x * sinAngle + square[i].y * cosAngle;
        rotated[i].x = centerX + static_cast<LONG>(rotatedX);
        rotated[i].y = centerY + static_cast<LONG>(rotatedY);
    }

    const COLORREF background = embedded ? RGB(18, 18, 24) : RGB(38, 30, 24);
    HBRUSH backgroundBrush = CreateSolidBrush(background);
    FillRect(hdc, &clientRect, backgroundBrush);
    DeleteObject(backgroundBrush);

    HPEN pen = CreatePen(PS_SOLID, 3, RGB(255, 196, 0));
    HBRUSH squareBrush = CreateSolidBrush(RGB(36, 140, 255));
    HGDIOBJ oldPen = SelectObject(hdc, pen);
    HGDIOBJ oldBrush = SelectObject(hdc, squareBrush);

    SetBkMode(hdc, TRANSPARENT);
    Polygon(hdc, rotated, 4);

    SelectObject(hdc, oldBrush);
    SelectObject(hdc, oldPen);
    DeleteObject(squareBrush);
    DeleteObject(pen);
}
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR, int)
{
    WNDCLASSEXW windowClass = {};
    windowClass.cbSize = sizeof(windowClass);
    windowClass.lpfnWndProc = WindowProc;
    windowClass.hInstance = instance;
    windowClass.lpszClassName = kWindowClassName;
    windowClass.hCursor = LoadCursorW(nullptr, IDC_ARROW);
    windowClass.hbrBackground = static_cast<HBRUSH>(GetStockObject(BLACK_BRUSH));

    if (RegisterClassExW(&windowClass) == 0)
    {
        MessageBoxW(nullptr, L"Failed to register window class.", kWindowTitle, MB_ICONERROR | MB_OK);
        return 1;
    }

    RECT primaryScreen = GetPrimaryScreenRect();
    const int initialWidth = std::max<int>(640, primaryScreen.right - primaryScreen.left);
    const int initialHeight = std::max<int>(360, primaryScreen.bottom - primaryScreen.top);

    HWND window = CreateWindowExW(
        WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
        kWindowClassName,
        kWindowTitle,
        WS_POPUP | WS_VISIBLE | WS_CLIPCHILDREN | WS_CLIPSIBLINGS,
        primaryScreen.left,
        primaryScreen.top,
        initialWidth,
        initialHeight,
        nullptr,
        nullptr,
        instance,
        nullptr);

    if (window == nullptr)
    {
        MessageBoxW(nullptr, L"Failed to create the main window.", kWindowTitle, MB_ICONERROR | MB_OK);
        return 1;
    }

    ApplyLayeredWindowSettings(window);
    ShowWindow(window, SW_SHOWNOACTIVATE);
    UpdateWindow(window);

    MSG message = {};
    while (GetMessageW(&message, nullptr, 0, 0) > 0)
    {
        TranslateMessage(&message);
        DispatchMessageW(&message);
    }

    return static_cast<int>(message.wParam);
}

LRESULT CALLBACK WindowProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam)
{
    AppState* appState = reinterpret_cast<AppState*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));

    switch (message)
    {
    case WM_CREATE:
    {
        auto* state = new AppState();
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(state));

        AttachToDesktopHost(hwnd, *state);
        SetTimer(hwnd, kAnimationTimerId, kFrameIntervalMs, nullptr);
        SetTimer(hwnd, kHostValidationTimerId, kHostValidationIntervalMs, nullptr);
        return 0;
    }
    case WM_TIMER:
        if (appState == nullptr)
        {
            return 0;
        }

        if (wParam == kAnimationTimerId)
        {
            appState->angleRadians += kRotationStepRadians;
            InvalidateRect(hwnd, nullptr, FALSE);
            return 0;
        }

        if (wParam == kHostValidationTimerId)
        {
            if (!IsHostRelationshipValid(*appState, hwnd))
            {
                AttachToDesktopHost(hwnd, *appState);
            }
            return 0;
        }
        return 0;
    case WM_DISPLAYCHANGE:
    case WM_DPICHANGED:
    case WM_SETTINGCHANGE:
        if (appState != nullptr)
        {
            AttachToDesktopHost(hwnd, *appState);
        }
        return 0;
    case WM_NCHITTEST:
        return HTTRANSPARENT;
    case WM_MOUSEACTIVATE:
        return MA_NOACTIVATE;
    case WM_ERASEBKGND:
        return 1;
    case WM_PAINT:
        if (appState != nullptr)
        {
            PAINTSTRUCT paint = {};
            HDC hdc = BeginPaint(hwnd, &paint);
            RECT clientRect = {};
            GetClientRect(hwnd, &clientRect);
            DrawRotatingSquare(hdc, clientRect, appState->angleRadians, appState->embedded);
            EndPaint(hwnd, &paint);
            return 0;
        }
        break;
    case WM_DESTROY:
        KillTimer(hwnd, kAnimationTimerId);
        KillTimer(hwnd, kHostValidationTimerId);
        delete appState;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcW(hwnd, message, wParam, lParam);
}
