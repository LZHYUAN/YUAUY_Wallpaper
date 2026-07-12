#pragma once

#include <d3d11.h>
#include <windows.h>

#include <atomic>
#include <thread>

class D3D11Renderer
{
public:
    D3D11Renderer();
    ~D3D11Renderer();

    bool Initialize(HWND window);
    void Start();
    void Stop();
    void Resize();

private:
    void RenderLoop();
    bool CreateRenderTarget();
    void ReleaseRenderTarget();
    void RenderFrame();
    void Cleanup();

    HWND window_ = nullptr;
    ID3D11Device* device_ = nullptr;
    ID3D11DeviceContext* context_ = nullptr;
    IDXGISwapChain* swapChain_ = nullptr;
    ID3D11RenderTargetView* renderTargetView_ = nullptr;
    std::thread renderThread_;
    std::atomic<bool> running_ = false;
    std::atomic<bool> resizePending_ = false;
    float phase_ = 0.0f;
};
