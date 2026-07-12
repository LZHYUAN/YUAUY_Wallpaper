#include "YUAUY_Wallpaper_DX.h"

#include "D3D11Renderer.h"

namespace
{
constexpr wchar_t kWindowClassName[] = L"YUAUYWallpaperWindow";
constexpr wchar_t kWindowTitle[] = L"YUAUY Wallpaper DX";

struct AppState
{
    D3D11Renderer renderer;
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

    HWND window = CreateWindowExW(
        WS_EX_LAYERED,
        kWindowClassName,
        kWindowTitle,
        0,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        nullptr,
        nullptr,
        instance,
        nullptr);

    if (window == nullptr)
    {
        MessageBoxW(nullptr, L"Failed to create the main window.", kWindowTitle, MB_ICONERROR | MB_OK);
        return 1;
    }

    SetWindowLongPtrW(window, GWL_STYLE, WS_VISIBLE | WS_CLIPSIBLINGS | WS_OVERLAPPED);
    SetLayeredWindowAttributes(window, 0, 0xFF, LWA_ALPHA);

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

        RECT primaryScreen = GetPrimaryScreenRect();
        const int width = primaryScreen.right - primaryScreen.left;
        const int height = primaryScreen.bottom - primaryScreen.top;

        HWND progman = FindWindowW(L"Progman", nullptr);
        if (progman != nullptr)
        {
            SendMessageW(progman, 0x052C, 0, 0);
        }

        HWND shellView = progman != nullptr ? FindWindowExW(progman, nullptr, L"SHELLDLL_DefView", nullptr) : nullptr;
        HWND insertAfter = shellView != nullptr ? shellView : HWND_BOTTOM;

        SetParent(hwnd, progman);

        SetWindowPos(
            hwnd,
            insertAfter,
            primaryScreen.left,
            primaryScreen.top,
            width,
            height,
            SWP_NOACTIVATE | SWP_SHOWWINDOW | SWP_FRAMECHANGED);

        if (!state->renderer.Initialize(hwnd))
        {
            MessageBoxW(hwnd, L"Failed to initialize D3D11.", kWindowTitle, MB_ICONERROR | MB_OK);
            delete state;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
            return -1;
        }

        state->renderer.Start();
        return 0;
    }
    case WM_SIZE:
        if (appState != nullptr && wParam != SIZE_MINIMIZED)
        {
            appState->renderer.Resize();
        }
        return 0;
    case WM_DESTROY:
        if (appState != nullptr)
        {
            appState->renderer.Stop();
            delete appState;
            SetWindowLongPtrW(hwnd, GWLP_USERDATA, 0);
        }
        PostQuitMessage(0);
        return 0;
    }

    return DefWindowProcW(hwnd, message, wParam, lParam);
}

