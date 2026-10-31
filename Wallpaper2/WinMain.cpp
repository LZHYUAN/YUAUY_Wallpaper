#include <windows.h>

#include "DxMain.cpp"

DxMain g_dxMain;


LRESULT CALLBACK WindowProc(HWND hWnd, UINT uMsg, WPARAM wParam, LPARAM lParam)
{
	switch (uMsg)
	{
	case WM_DESTROY:
		PostQuitMessage(0);
		break;
	case WM_SIZE: 
		g_dxMain.SetViewports(LOWORD(lParam), HIWORD(lParam));
		return DefWindowProc(hWnd, uMsg, wParam, lParam);
	default:
		return DefWindowProc(hWnd, uMsg, wParam, lParam);
	}
	return 0;
}


int WINAPI wWinMain(HINSTANCE hInstance, HINSTANCE hPrevInstance, PWSTR pCmdLine, int nCmdShow)
{
	WNDCLASSEX wc = {
		sizeof(WNDCLASSEX),// cbSize
		CS_HREDRAW | CS_VREDRAW,// style
		WindowProc,// lpfnWndProc
		NULL,// cbClsExtra
		NULL,// cbWndExtra
		NULL,// hInstance
		NULL,// hIcon
		NULL,//LoadCursor(NULL, IDC_ARROW),// hCursor
		NULL,// hbrBackground
		NULL,// lpszMenuName
		L"lpszClassName",// lpszClassName
		NULL// hIconSm
	};
	RegisterClassEx(&wc);


	HWND hWnd = CreateWindowEx(
		WS_EX_NOREDIRECTIONBITMAP,// dwExStyle
		wc.lpszClassName,// lpClassName
		L"lpWindowName",// lpWindowName
		WS_OVERLAPPEDWINDOW,// dwStyle
		0,// X
		0,// Y
		500,// nWidth
		500,// nHeight
		NULL,// hWndParent
		NULL,// hMenu
		hInstance,// hInstance
		NULL	// lpPara
	);

	g_dxMain.CreateD3D(hWnd);	
	g_dxMain.SetViewports(500, 500);
	
	ShowWindow(hWnd, nCmdShow);

	MSG msg = { 0 };
	while (WM_QUIT != msg.message)
	{
		if (PeekMessage(&msg, NULL, 0, 0, PM_REMOVE))
		{
			TranslateMessage(&msg);
			DispatchMessage(&msg);
		}
		else
		{
			g_dxMain.Render();
		}
	}


	return 0;


}