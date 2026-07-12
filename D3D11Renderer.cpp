#include "D3D11Renderer.h"

#include <cmath>
#include <thread>

namespace
{
constexpr UINT kVSyncInterval = 1;
}

D3D11Renderer::D3D11Renderer() = default;

D3D11Renderer::~D3D11Renderer()
{
    Stop();
    Cleanup();
}

bool D3D11Renderer::Initialize(HWND window)
{
    window_ = window;

    DXGI_SWAP_CHAIN_DESC swapChainDesc = {};
    swapChainDesc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    swapChainDesc.SampleDesc.Count = 1;
    swapChainDesc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    swapChainDesc.BufferCount = 2;
    swapChainDesc.OutputWindow = window_;
    swapChainDesc.Windowed = TRUE;
    swapChainDesc.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    UINT createDeviceFlags = D3D11_CREATE_DEVICE_SINGLETHREADED;
#if defined(_DEBUG)
    createDeviceFlags |= D3D11_CREATE_DEVICE_DEBUG;
#endif

    const D3D_FEATURE_LEVEL featureLevels[] = {
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    };

    D3D_FEATURE_LEVEL featureLevel = D3D_FEATURE_LEVEL_11_0;
    const HRESULT hr = D3D11CreateDeviceAndSwapChain(
        nullptr,
        D3D_DRIVER_TYPE_HARDWARE,
        nullptr,
        createDeviceFlags,
        featureLevels,
        static_cast<UINT>(sizeof(featureLevels) / sizeof(featureLevels[0])),
        D3D11_SDK_VERSION,
        &swapChainDesc,
        &swapChain_,
        &device_,
        &featureLevel,
        &context_);

    if (FAILED(hr))
    {
        Cleanup();
        return false;
    }

    if (!CreateRenderTarget())
    {
        Cleanup();
        return false;
    }

    return true;
}

void D3D11Renderer::Start()
{
    if (running_.exchange(true))
    {
        return;
    }

    renderThread_ = std::thread(&D3D11Renderer::RenderLoop, this);
}

void D3D11Renderer::Stop()
{
    if (!running_.exchange(false))
    {
        return;
    }

    if (renderThread_.joinable())
    {
        renderThread_.join();
    }
}

void D3D11Renderer::Resize()
{
    resizePending_.store(true);
}

void D3D11Renderer::RenderLoop()
{
    while (running_.load())
    {
        if (resizePending_.exchange(false) && swapChain_ != nullptr)
        {
            ReleaseRenderTarget();
            if (SUCCEEDED(swapChain_->ResizeBuffers(0, 0, 0, DXGI_FORMAT_UNKNOWN, 0)))
            {
                CreateRenderTarget();
            }
        }

        RenderFrame();
    }
}

bool D3D11Renderer::CreateRenderTarget()
{
    ReleaseRenderTarget();

    if (swapChain_ == nullptr || device_ == nullptr || context_ == nullptr)
    {
        return false;
    }

    ID3D11Texture2D* backBuffer = nullptr;
    HRESULT hr = swapChain_->GetBuffer(0, __uuidof(ID3D11Texture2D), reinterpret_cast<void**>(&backBuffer));
    if (FAILED(hr))
    {
        return false;
    }

    hr = device_->CreateRenderTargetView(backBuffer, nullptr, &renderTargetView_);
    backBuffer->Release();
    if (FAILED(hr))
    {
        return false;
    }

    RECT clientRect = {};
    GetClientRect(window_, &clientRect);
    D3D11_VIEWPORT viewport = {};
    viewport.TopLeftX = 0.0f;
    viewport.TopLeftY = 0.0f;
    viewport.Width = static_cast<float>(clientRect.right - clientRect.left);
    viewport.Height = static_cast<float>(clientRect.bottom - clientRect.top);
    viewport.MinDepth = 0.0f;
    viewport.MaxDepth = 1.0f;
    context_->RSSetViewports(1, &viewport);

    return true;
}

void D3D11Renderer::ReleaseRenderTarget()
{
    if (renderTargetView_ != nullptr)
    {
        renderTargetView_->Release();
        renderTargetView_ = nullptr;
    }
}

void D3D11Renderer::RenderFrame()
{
    if (context_ == nullptr || renderTargetView_ == nullptr || swapChain_ == nullptr)
    {
        std::this_thread::sleep_for(std::chrono::milliseconds(16));
        return;
    }

    phase_ += 0.02f;
    const float clearColor[4] = {
        0.08f + 0.04f * std::sin(phase_),
        0.16f + 0.08f * std::sin(phase_ * 1.7f),
        0.28f + 0.12f * std::sin(phase_ * 0.8f),
        1.0f,
    };

    context_->OMSetRenderTargets(1, &renderTargetView_, nullptr);
    context_->ClearRenderTargetView(renderTargetView_, clearColor);
    swapChain_->Present(kVSyncInterval, 0);
}

void D3D11Renderer::Cleanup()
{
    ReleaseRenderTarget();

    if (swapChain_ != nullptr)
    {
        swapChain_->Release();
        swapChain_ = nullptr;
    }

    if (context_ != nullptr)
    {
        context_->Release();
        context_ = nullptr;
    }

    if (device_ != nullptr)
    {
        device_->Release();
        device_ = nullptr;
    }

    window_ = nullptr;
    resizePending_.store(false);
}
