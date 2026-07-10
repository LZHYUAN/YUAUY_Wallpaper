#include "YUAUY_Wallpaper_DX.h"

#include <algorithm>
#include <cmath>

namespace
{
constexpr wchar_t kWindowClassName[] = L"YUAUYWallpaperWindow";
constexpr wchar_t kWindowTitle[] = L"YUAUY Wallpaper DX";
constexpr UINT_PTR kAnimationTimerId = 1;
constexpr UINT kFrameIntervalMs = 16;
constexpr double kRotationStepRadians = 0.04;
constexpr UINT kSpawnWorkerMessage = 0x052C;

struct AppState
{
    double angleRadians = 0.0;
    bool embedded = false;
};

RECT GetPrimaryScreenRect()
{
    RECT rect = {};
    rect.left = 0;
    rect.top = 0;
    rect.right = GetSystemMetrics(SM_CXSCREEN);
    rect.bottom = GetSystemMetrics(SM_CYSCREEN);
    return rect;
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
    exStyle |= WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW;
    exStyle &= ~static_cast<LONG_PTR>(WS_EX_APPWINDOW);
    SetWindowLongPtrW(window, GWL_EXSTYLE, exStyle);

    ApplyLayeredWindowSettings(window);
}

void DrawRotatingSquare(HDC hdc, const RECT& clientRect, double angleRadians)
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

    HBRUSH backgroundBrush = CreateSolidBrush(RGB(18, 18, 24));
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

bool EmbedWindowIntoProgman(HWND window)
{
    HWND progman = FindWindowW(L"Progman", nullptr);
    if (progman == nullptr)
    {
        return false;
    }

    DWORD_PTR messageResult = 0;
    SendMessageTimeoutW(
        progman,
        kSpawnWorkerMessage,
        0,
        0,
        SMTO_NORMAL,
        1000,
        &messageResult);

    HWND defView = FindWindowExW(progman, nullptr, L"SHELLDLL_DefView", nullptr);
    HWND workerW = FindWindowExW(nullptr, progman, L"WorkerW", nullptr);

    NormalizeAsWallpaperChild(window);

    SetLastError(0);
    if (SetParent(window, progman) == nullptr && GetLastError() != 0)
    {
        return false;
    }

    RECT bounds = GetPrimaryScreenRect();
    const int width = bounds.right - bounds.left;
    const int height = bounds.bottom - bounds.top;
    HWND insertAfter = defView != nullptr ? defView : HWND_BOTTOM;

    UINT flags = SWP_SHOWWINDOW | SWP_NOACTIVATE | SWP_FRAMECHANGED;
    if (!SetWindowPos(window, insertAfter, 0, 0, width, height, flags))
    {
        return false;
    }

    if (workerW != nullptr)
    {
        SetWindowPos(workerW, window, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
    }

    ApplyLayeredWindowSettings(window);
    InvalidateRect(window, nullptr, FALSE);
    UpdateWindow(window);
    return true;
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
        WS_EX_LAYERED | WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW,
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
        state->embedded = EmbedWindowIntoProgman(hwnd);
        SetTimer(hwnd, kAnimationTimerId, kFrameIntervalMs, nullptr);
        return 0;
    }
    case WM_TIMER:
        if (appState != nullptr && wParam == kAnimationTimerId)
        {
            appState->angleRadians += kRotationStepRadians;
            InvalidateRect(hwnd, nullptr, FALSE);
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
            DrawRotatingSquare(hdc, clientRect, appState->angleRadians);
            EndPaint(hwnd, &paint);
            return 0;
        }
        break;
    case WM_DESTROY:
        KillTimer(hwnd, kAnimationTimerId);
        delete appState;
        SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcW(hwnd, message, wParam, lParam);
}
