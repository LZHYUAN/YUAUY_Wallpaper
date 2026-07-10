#pragma once

#ifndef NOMINMAX
#define NOMINMAX
#endif

#include <windows.h>

LRESULT CALLBACK WindowProc(HWND hwnd, UINT message, WPARAM wParam, LPARAM lParam);
