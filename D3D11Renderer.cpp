#include "D3D11Renderer.h"

#include <d3dcompiler.h>

#include <algorithm>
#include <array>
#include <chrono>
#include <cmath>
#include <cstdint>
#include <cstring>
#include <cwctype>
#include <thread>
#include <utility>

namespace
{
using Microsoft::WRL::ComPtr;

constexpr UINT kVSyncInterval = 1;
constexpr int kFrameDelayMs = 40;
constexpr float kScaleReduction = 0.2f;
constexpr wchar_t kImageFolder[] = L"D:\\YUAUY\\Pictures\\Dropbox\\\u684C\u5E03";
constexpr char kVertexShaderSource[] = R"(
cbuffer Constants : register(b0)
{
    float2 screenSize;
    float4 tint;
};

struct VSInput
{
    float2 position : POSITION;
    float2 texcoord : TEXCOORD0;
};

struct VSOutput
{
    float4 position : SV_POSITION;
    float2 texcoord : TEXCOORD0;
};

VSOutput main(VSInput input)
{
    VSOutput output;
    float2 ndc;
    ndc.x = (input.position.x / screenSize.x) * 2.0f - 1.0f;
    ndc.y = 1.0f - (input.position.y / screenSize.y) * 2.0f;
    output.position = float4(ndc, 0.0f, 1.0f);
    output.texcoord = input.texcoord;
    return output;
}
)";

constexpr char kPixelShaderSource[] = R"(
cbuffer Constants : register(b0)
{
    float2 screenSize;
    float4 tint;
};

Texture2D sourceTexture : register(t0);
SamplerState sourceSampler : register(s0);

float4 main(float4 position : SV_POSITION, float2 texcoord : TEXCOORD0) : SV_TARGET
{
    return sourceTexture.Sample(sourceSampler, texcoord) * tint;
}
)";
}

bool D3D11Renderer::Texture::IsValid() const
{
    return srv != nullptr && width > 0 && height > 0;
}

bool D3D11Renderer::RenderTexture::IsValid() const
{
    return texture != nullptr && rtv != nullptr && srv != nullptr && width > 0 && height > 0;
}

D3D11Renderer::D3D11Renderer()
    : randomEngine_(static_cast<std::mt19937::result_type>(
          std::chrono::high_resolution_clock::now().time_since_epoch().count()))
{
}

D3D11Renderer::~D3D11Renderer()
{
    Stop();
    Cleanup();
}

bool D3D11Renderer::Initialize(HWND window)
{
    window_ = window;

    RECT clientRect = {};
    GetClientRect(window_, &clientRect);
    screenWidth_ = std::max(1L, clientRect.right - clientRect.left);
    screenHeight_ = std::max(1L, clientRect.bottom - clientRect.top);

    DXGI_SWAP_CHAIN_DESC swapChainDesc = {};
    swapChainDesc.BufferDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    swapChainDesc.SampleDesc.Count = 1;
    swapChainDesc.BufferUsage = DXGI_USAGE_RENDER_TARGET_OUTPUT;
    swapChainDesc.BufferCount = 2;
    swapChainDesc.OutputWindow = window_;
    swapChainDesc.Windowed = TRUE;
    swapChainDesc.SwapEffect = DXGI_SWAP_EFFECT_DISCARD;

    const D3D_FEATURE_LEVEL featureLevels[] = {
        D3D_FEATURE_LEVEL_11_1,
        D3D_FEATURE_LEVEL_11_0,
        D3D_FEATURE_LEVEL_10_1,
        D3D_FEATURE_LEVEL_10_0,
    };

    UINT createDeviceFlags = 0;
#if defined(_DEBUG)
    createDeviceFlags |= D3D11_CREATE_DEVICE_DEBUG;
#endif

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

    if (!CreateBackBufferTarget() || !CreateDeviceResources() || !CreateSceneResources())
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
    CoInitializeEx(nullptr, COINIT_MULTITHREADED);
    CreateWicFactory();
    CollectImagePaths();
    PerformStartupFade();
    RunTransitionLoop();
    wicFactory_.Reset();
    CoUninitialize();
}

bool D3D11Renderer::CreateBackBufferTarget()
{
    ReleaseBackBufferTarget();

    ComPtr<ID3D11Texture2D> backBuffer;
    if (FAILED(swapChain_->GetBuffer(0, IID_PPV_ARGS(&backBuffer))))
    {
        return false;
    }

    if (FAILED(device_->CreateRenderTargetView(backBuffer.Get(), nullptr, &backBufferRtv_)))
    {
        return false;
    }

    D3D11_VIEWPORT viewport = {};
    viewport.TopLeftX = 0.0f;
    viewport.TopLeftY = 0.0f;
    viewport.Width = static_cast<float>(screenWidth_);
    viewport.Height = static_cast<float>(screenHeight_);
    viewport.MinDepth = 0.0f;
    viewport.MaxDepth = 1.0f;
    context_->RSSetViewports(1, &viewport);
    return true;
}

bool D3D11Renderer::CreateDeviceResources()
{
    ComPtr<ID3DBlob> vertexShaderBlob;
    ComPtr<ID3DBlob> pixelShaderBlob;

    if (FAILED(D3DCompile(
            kVertexShaderSource,
            sizeof(kVertexShaderSource) - 1,
            nullptr,
            nullptr,
            nullptr,
            "main",
            "vs_4_0",
            0,
            0,
            &vertexShaderBlob,
            nullptr)))
    {
        return false;
    }

    if (FAILED(D3DCompile(
            kPixelShaderSource,
            sizeof(kPixelShaderSource) - 1,
            nullptr,
            nullptr,
            nullptr,
            "main",
            "ps_4_0",
            0,
            0,
            &pixelShaderBlob,
            nullptr)))
    {
        return false;
    }

    if (FAILED(device_->CreateVertexShader(
            vertexShaderBlob->GetBufferPointer(),
            vertexShaderBlob->GetBufferSize(),
            nullptr,
            &vertexShader_)))
    {
        return false;
    }

    if (FAILED(device_->CreatePixelShader(
            pixelShaderBlob->GetBufferPointer(),
            pixelShaderBlob->GetBufferSize(),
            nullptr,
            &pixelShader_)))
    {
        return false;
    }

    const D3D11_INPUT_ELEMENT_DESC inputLayout[] = {
        { "POSITION", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 0, D3D11_INPUT_PER_VERTEX_DATA, 0 },
        { "TEXCOORD", 0, DXGI_FORMAT_R32G32_FLOAT, 0, 8, D3D11_INPUT_PER_VERTEX_DATA, 0 },
    };

    if (FAILED(device_->CreateInputLayout(
            inputLayout,
            static_cast<UINT>(sizeof(inputLayout) / sizeof(inputLayout[0])),
            vertexShaderBlob->GetBufferPointer(),
            vertexShaderBlob->GetBufferSize(),
            &inputLayout_)))
    {
        return false;
    }

    D3D11_BUFFER_DESC vertexBufferDesc = {};
    vertexBufferDesc.ByteWidth = static_cast<UINT>(sizeof(Vertex) * 6);
    vertexBufferDesc.Usage = D3D11_USAGE_DYNAMIC;
    vertexBufferDesc.BindFlags = D3D11_BIND_VERTEX_BUFFER;
    vertexBufferDesc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    if (FAILED(device_->CreateBuffer(&vertexBufferDesc, nullptr, &vertexBuffer_)))
    {
        return false;
    }

    D3D11_BUFFER_DESC constantBufferDesc = {};
    constantBufferDesc.ByteWidth = sizeof(ShaderConstants);
    constantBufferDesc.Usage = D3D11_USAGE_DYNAMIC;
    constantBufferDesc.BindFlags = D3D11_BIND_CONSTANT_BUFFER;
    constantBufferDesc.CPUAccessFlags = D3D11_CPU_ACCESS_WRITE;
    if (FAILED(device_->CreateBuffer(&constantBufferDesc, nullptr, &constantBuffer_)))
    {
        return false;
    }

    D3D11_SAMPLER_DESC samplerDesc = {};
    samplerDesc.Filter = D3D11_FILTER_MIN_MAG_MIP_POINT;
    samplerDesc.AddressU = D3D11_TEXTURE_ADDRESS_CLAMP;
    samplerDesc.AddressV = D3D11_TEXTURE_ADDRESS_CLAMP;
    samplerDesc.AddressW = D3D11_TEXTURE_ADDRESS_CLAMP;
    samplerDesc.MinLOD = 0.0f;
    samplerDesc.MaxLOD = D3D11_FLOAT32_MAX;
    if (FAILED(device_->CreateSamplerState(&samplerDesc, &samplerState_)))
    {
        return false;
    }

    D3D11_BLEND_DESC blendDesc = {};
    blendDesc.RenderTarget[0].BlendEnable = TRUE;
    blendDesc.RenderTarget[0].SrcBlend = D3D11_BLEND_SRC_ALPHA;
    blendDesc.RenderTarget[0].DestBlend = D3D11_BLEND_INV_SRC_ALPHA;
    blendDesc.RenderTarget[0].BlendOp = D3D11_BLEND_OP_ADD;
    blendDesc.RenderTarget[0].SrcBlendAlpha = D3D11_BLEND_ONE;
    blendDesc.RenderTarget[0].DestBlendAlpha = D3D11_BLEND_INV_SRC_ALPHA;
    blendDesc.RenderTarget[0].BlendOpAlpha = D3D11_BLEND_OP_ADD;
    blendDesc.RenderTarget[0].RenderTargetWriteMask = D3D11_COLOR_WRITE_ENABLE_ALL;
    if (FAILED(device_->CreateBlendState(&blendDesc, &blendState_)))
    {
        return false;
    }

    return CreateWhiteTexture();
}

bool D3D11Renderer::CreateSceneResources()
{
    if (!CreateRenderTexture(screenWidth_, screenHeight_, baseTexture_) ||
        !CreateRenderTexture(screenWidth_, screenHeight_, scratchTexture_))
    {
        return false;
    }

    const float clearColor[4] = { 0.0f, 0.0f, 0.0f, 1.0f };
    context_->ClearRenderTargetView(baseTexture_.rtv.Get(), clearColor);
    context_->ClearRenderTargetView(scratchTexture_.rtv.Get(), clearColor);
    return true;
}

bool D3D11Renderer::CreateRenderTexture(int width, int height, RenderTexture& target)
{
    target = {};

    D3D11_TEXTURE2D_DESC textureDesc = {};
    textureDesc.Width = static_cast<UINT>(width);
    textureDesc.Height = static_cast<UINT>(height);
    textureDesc.MipLevels = 1;
    textureDesc.ArraySize = 1;
    textureDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    textureDesc.SampleDesc.Count = 1;
    textureDesc.Usage = D3D11_USAGE_DEFAULT;
    textureDesc.BindFlags = D3D11_BIND_RENDER_TARGET | D3D11_BIND_SHADER_RESOURCE;

    if (FAILED(device_->CreateTexture2D(&textureDesc, nullptr, &target.texture)))
    {
        return false;
    }

    if (FAILED(device_->CreateRenderTargetView(target.texture.Get(), nullptr, &target.rtv)))
    {
        target = {};
        return false;
    }

    if (FAILED(device_->CreateShaderResourceView(target.texture.Get(), nullptr, &target.srv)))
    {
        target = {};
        return false;
    }

    target.width = width;
    target.height = height;
    return true;
}

bool D3D11Renderer::CreateTextureFromFile(const std::filesystem::path& path, Texture& texture)
{
    if (wicFactory_ == nullptr)
    {
        return false;
    }

    ComPtr<IWICBitmapDecoder> decoder;
    if (FAILED(wicFactory_->CreateDecoderFromFilename(
            path.c_str(),
            nullptr,
            GENERIC_READ,
            WICDecodeMetadataCacheOnDemand,
            &decoder)))
    {
        return false;
    }

    ComPtr<IWICBitmapFrameDecode> frame;
    if (FAILED(decoder->GetFrame(0, &frame)))
    {
        return false;
    }

    UINT sourceWidth = 0;
    UINT sourceHeight = 0;
    frame->GetSize(&sourceWidth, &sourceHeight);
    if (sourceWidth == 0 || sourceHeight == 0)
    {
        return false;
    }

    const float irs = static_cast<float>(sourceWidth) / static_cast<float>(sourceHeight);
    const float srs = static_cast<float>(screenWidth_) / static_cast<float>(screenHeight_);
    const float rsx = std::fabs(irs - srs) * 0.1f + 0.8f;

    int targetWidth = 0;
    int targetHeight = 0;
    if (irs > srs)
    {
        targetWidth = static_cast<int>(screenWidth_ * rsx) + 20;
        targetHeight = static_cast<int>(screenWidth_ * rsx / static_cast<float>(sourceWidth) * static_cast<float>(sourceHeight)) + 20;
    }
    else
    {
        targetWidth = static_cast<int>(screenHeight_ * rsx / static_cast<float>(sourceHeight) * static_cast<float>(sourceWidth)) + 20;
        targetHeight = static_cast<int>(screenHeight_ * rsx) + 20;
    }

    targetWidth = std::max(targetWidth, 20);
    targetHeight = std::max(targetHeight, 20);
    const UINT scaledWidth = static_cast<UINT>(std::max(1, targetWidth - 20));
    const UINT scaledHeight = static_cast<UINT>(std::max(1, targetHeight - 20));

    ComPtr<IWICBitmapScaler> scaler;
    if (FAILED(wicFactory_->CreateBitmapScaler(&scaler)))
    {
        return false;
    }

    if (FAILED(scaler->Initialize(frame.Get(), scaledWidth, scaledHeight, WICBitmapInterpolationModeNearestNeighbor)))
    {
        return false;
    }

    ComPtr<IWICFormatConverter> converter;
    if (FAILED(wicFactory_->CreateFormatConverter(&converter)))
    {
        return false;
    }

    if (FAILED(converter->Initialize(
            scaler.Get(),
            GUID_WICPixelFormat32bppRGBA,
            WICBitmapDitherTypeNone,
            nullptr,
            0.0,
            WICBitmapPaletteTypeCustom)))
    {
        return false;
    }

    std::vector<std::uint8_t> scaledPixels(static_cast<size_t>(scaledWidth) * scaledHeight * 4);
    if (FAILED(converter->CopyPixels(nullptr, scaledWidth * 4, static_cast<UINT>(scaledPixels.size()), scaledPixels.data())))
    {
        return false;
    }

    std::vector<std::uint8_t> texturePixels(static_cast<size_t>(targetWidth) * targetHeight * 4, 255);
    for (UINT row = 0; row < scaledHeight; ++row)
    {
        std::uint8_t* destinationRow = texturePixels.data() + (static_cast<size_t>(row) + 10) * targetWidth * 4 + 10 * 4;
        const std::uint8_t* sourceRow = scaledPixels.data() + static_cast<size_t>(row) * scaledWidth * 4;
        std::memcpy(destinationRow, sourceRow, static_cast<size_t>(scaledWidth) * 4);
    }

    return CreateTextureFromPixels(texturePixels.data(), targetWidth, targetHeight, texture);
}

bool D3D11Renderer::CreateTextureFromPixels(const std::uint8_t* pixels, int width, int height, Texture& texture)
{
    texture = {};

    D3D11_TEXTURE2D_DESC textureDesc = {};
    textureDesc.Width = static_cast<UINT>(width);
    textureDesc.Height = static_cast<UINT>(height);
    textureDesc.MipLevels = 1;
    textureDesc.ArraySize = 1;
    textureDesc.Format = DXGI_FORMAT_R8G8B8A8_UNORM;
    textureDesc.SampleDesc.Count = 1;
    textureDesc.Usage = D3D11_USAGE_DEFAULT;
    textureDesc.BindFlags = D3D11_BIND_SHADER_RESOURCE;

    D3D11_SUBRESOURCE_DATA initialData = {};
    initialData.pSysMem = pixels;
    initialData.SysMemPitch = static_cast<UINT>(width * 4);

    if (FAILED(device_->CreateTexture2D(&textureDesc, &initialData, &texture.texture)))
    {
        return false;
    }

    if (FAILED(device_->CreateShaderResourceView(texture.texture.Get(), nullptr, &texture.srv)))
    {
        texture = {};
        return false;
    }

    texture.width = width;
    texture.height = height;
    return true;
}

bool D3D11Renderer::CreateWhiteTexture()
{
    const std::array<std::uint8_t, 4> whitePixel = { 255, 255, 255, 255 };
    return CreateTextureFromPixels(whitePixel.data(), 1, 1, whiteTexture_);
}

bool D3D11Renderer::CreateWicFactory()
{
    if (wicFactory_ != nullptr)
    {
        return true;
    }

    return SUCCEEDED(CoCreateInstance(
        CLSID_WICImagingFactory,
        nullptr,
        CLSCTX_INPROC_SERVER,
        IID_PPV_ARGS(&wicFactory_)));
}

void D3D11Renderer::ReleaseBackBufferTarget()
{
    backBufferRtv_.Reset();
}

void D3D11Renderer::Cleanup()
{
    lastTexture_ = {};
    whiteTexture_ = {};
    baseTexture_ = {};
    scratchTexture_ = {};
    wicFactory_.Reset();
    blendState_.Reset();
    samplerState_.Reset();
    constantBuffer_.Reset();
    vertexBuffer_.Reset();
    inputLayout_.Reset();
    pixelShader_.Reset();
    vertexShader_.Reset();
    ReleaseBackBufferTarget();
    swapChain_.Reset();
    context_.Reset();
    device_.Reset();
    window_ = nullptr;
    imagePaths_.clear();
    imageCursor_ = 0;
    first_ = true;
    firstCount_ = 0;
    screenWidth_ = 0;
    screenHeight_ = 0;
    transitionX_ = 0.0f;
    transitionY_ = 0.0f;
    transitionRotationDegrees_ = 0.0f;
    resizePending_.store(false);
}

void D3D11Renderer::HandleResize()
{
    RECT clientRect = {};
    GetClientRect(window_, &clientRect);
    const int newWidth = std::max(1L, clientRect.right - clientRect.left);
    const int newHeight = std::max(1L, clientRect.bottom - clientRect.top);
    if (newWidth == screenWidth_ && newHeight == screenHeight_)
    {
        return;
    }

    screenWidth_ = newWidth;
    screenHeight_ = newHeight;
    ReleaseBackBufferTarget();

    if (SUCCEEDED(swapChain_->ResizeBuffers(0, 0, 0, DXGI_FORMAT_UNKNOWN, 0)))
    {
        CreateBackBufferTarget();
        CreateSceneResources();
        lastTexture_ = {};
        first_ = true;
        firstCount_ = 0;
        CollectImagePaths();
    }
}

void D3D11Renderer::PerformStartupFade()
{
    for (int i = 1; i < 50 && running_.load(); ++i)
    {
        BeginRenderTarget(backBufferRtv_.Get(), true);
        const float tint[4] = { 0.0f, 0.0f, 0.0f, static_cast<float>(i) / 255.0f };
        DrawRectangle(0.0f, 0.0f, static_cast<float>(screenWidth_), static_cast<float>(screenHeight_), tint);
        swapChain_->Present(kVSyncInterval, 0);
    }
}

void D3D11Renderer::RunTransitionLoop()
{
    while (running_.load())
    {
        if (resizePending_.exchange(false))
        {
            HandleResize();
        }

#if defined(_DEBUG)
        const int waitMs = first_ ? 0 : 1000;
#else
        const int waitMs = first_ ? 0 : 30000;
#endif

        if (!WaitInterruptible(waitMs))
        {
            break;
        }

        Texture nextTexture;
        if (!LoadNextTexture(nextTexture))
        {
            RenderStaticScene();
            swapChain_->Present(kVSyncInterval, 0);
            if (!WaitInterruptible(1000))
            {
                break;
            }
            continue;
        }

        if (!lastTexture_.IsValid())
        {
            lastTexture_ = nextTexture;
        }

        const float screenWidth = static_cast<float>(screenWidth_);
        const float screenHeight = static_cast<float>(screenHeight_);
        std::uniform_int_distribution<int> xDistribution(0, std::max(0, static_cast<int>(screenWidth * 0.4f) - 1));
        std::uniform_int_distribution<int> yDistribution(0, std::max(0, static_cast<int>(screenHeight * 0.4f) - 1));
        std::uniform_int_distribution<int> rotationDistribution(-20, 19);

        float x = static_cast<float>(xDistribution(randomEngine_));
        if (x < screenWidth * 0.2f)
        {
            x -= screenWidth * 0.4f;
        }

        float y = static_cast<float>(yDistribution(randomEngine_));
        if (y < screenHeight * 0.2f)
        {
            y -= screenHeight * 0.4f;
        }

        const float rotationDegrees = static_cast<float>(rotationDistribution(randomEngine_));
        transitionX_ = x;
        transitionY_ = y;
        transitionRotationDegrees_ = rotationDegrees;

        const Texture previousTexture = lastTexture_;
        const float timeStep = first_ ? 1.0f : 0.03f;
        auto nextTime = std::chrono::steady_clock::now() + std::chrono::milliseconds(kFrameDelayMs);
        for (float time = 0.0f; time <= 1.0001f && running_.load(); time += timeStep)
        {
            if (resizePending_.exchange(false))
            {
                HandleResize();
            }

            RenderTransitionFrame(nextTexture, std::min(time, 1.0f));
            swapChain_->Present(kVSyncInterval, 0);
            std::this_thread::sleep_until(nextTime);
            nextTime += std::chrono::milliseconds(kFrameDelayMs);
        }

        CommitTransitionToBase(previousTexture, x, y, rotationDegrees, kScaleReduction);
        lastTexture_ = nextTexture;
        RenderStaticScene();
        swapChain_->Present(kVSyncInterval, 0);

        if (first_ && firstCount_ < 10)
        {
            ++firstCount_;
        }
        else
        {
            first_ = false;
        }
    }
}

bool D3D11Renderer::LoadNextTexture(Texture& texture)
{
    if (imagePaths_.empty())
    {
        CollectImagePaths();
    }

    if (imagePaths_.empty())
    {
        return false;
    }

    while (running_.load())
    {
        if (imageCursor_ >= imagePaths_.size())
        {
            ShuffleImagePaths();
        }

        if (CreateTextureFromFile(imagePaths_[imageCursor_++], texture))
        {
            return true;
        }
    }

    return false;
}

void D3D11Renderer::CollectImagePaths()
{
    imagePaths_.clear();
    imageCursor_ = 0;

    const std::filesystem::path folder(kImageFolder);
    if (!std::filesystem::exists(folder))
    {
        return;
    }

    for (const auto& entry : std::filesystem::directory_iterator(folder))
    {
        if (!entry.is_regular_file())
        {
            continue;
        }

        auto extension = entry.path().extension().wstring();
        std::transform(extension.begin(), extension.end(), extension.begin(), ::towlower);
        if (extension == L".png" || extension == L".jpg" || extension == L".jpeg")
        {
            imagePaths_.push_back(entry.path());
        }
    }

    ShuffleImagePaths();
}

void D3D11Renderer::ShuffleImagePaths()
{
    if (imagePaths_.empty())
    {
        imageCursor_ = 0;
        return;
    }

    std::shuffle(imagePaths_.begin(), imagePaths_.end(), randomEngine_);
    imageCursor_ = 0;
}

void D3D11Renderer::RenderStaticScene()
{
    BeginRenderTarget(backBufferRtv_.Get(), true);

    const float opaque[4] = { 1.0f, 1.0f, 1.0f, 1.0f };
    if (baseTexture_.IsValid())
    {
        DrawTexture(
            baseTexture_.srv.Get(),
            static_cast<float>(screenWidth_) * 0.5f,
            static_cast<float>(screenHeight_) * 0.5f,
            static_cast<float>(screenWidth_),
            static_cast<float>(screenHeight_),
            0.0f,
            opaque);
    }

    if (lastTexture_.IsValid())
    {
        DrawTexture(
            lastTexture_.srv.Get(),
            static_cast<float>(screenWidth_) * 0.5f,
            static_cast<float>(screenHeight_) * 0.5f,
            static_cast<float>(lastTexture_.width),
            static_cast<float>(lastTexture_.height),
            0.0f,
            opaque);
    }
}

void D3D11Renderer::RenderTransitionFrame(const Texture& nextTexture, float time)
{
    BeginRenderTarget(backBufferRtv_.Get(), true);

    const float screenCenterX = static_cast<float>(screenWidth_) * 0.5f;
    const float screenCenterY = static_cast<float>(screenHeight_) * 0.5f;
    const float opaque[4] = { 1.0f, 1.0f, 1.0f, 1.0f };

    DrawTexture(
        baseTexture_.srv.Get(),
        screenCenterX,
        screenCenterY,
        static_cast<float>(screenWidth_),
        static_cast<float>(screenHeight_),
        0.0f,
        opaque);

    const float timeA = EvaluateTimeA(time);
    const float oldCenterX = screenCenterX + transitionX_ * timeA;
    const float oldCenterY = screenCenterY + transitionY_ * timeA;
    const float oldRotation = transitionRotationDegrees_ * timeA;
    const float oldScale = 1.0f - kScaleReduction * timeA;
    const float oldWidth = static_cast<float>(lastTexture_.width) * oldScale;
    const float oldHeight = static_cast<float>(lastTexture_.height) * oldScale;
    const float blackTint[4] = { 0.0f, 0.0f, 0.0f, timeA * 175.0f / 255.0f };

    if (lastTexture_.IsValid())
    {
        DrawTexture(lastTexture_.srv.Get(), oldCenterX, oldCenterY, oldWidth, oldHeight, oldRotation, opaque);
        DrawTexture(whiteTexture_.srv.Get(), oldCenterX, oldCenterY, oldWidth, oldHeight, oldRotation, blackTint);
    }

    const float nextLeft = screenCenterX - static_cast<float>(nextTexture.width) * 0.5f;
    const float nextTop = screenCenterY - static_cast<float>(nextTexture.height) * 0.5f;

    if (time >= 0.75f)
    {
        DrawTexture(
            nextTexture.srv.Get(),
            screenCenterX,
            screenCenterY,
            static_cast<float>(nextTexture.width),
            static_cast<float>(nextTexture.height),
            0.0f,
            opaque);
    }
    else if (time >= 0.5f)
    {
        const float timeB = EvaluateRevealPhaseA(time);
        DrawTexture(
            nextTexture.srv.Get(),
            screenCenterX,
            screenCenterY,
            static_cast<float>(nextTexture.width),
            static_cast<float>(nextTexture.height),
            0.0f,
            opaque);

        const float maskTop = screenCenterY + static_cast<float>(nextTexture.height) * (timeB - 0.5f);
        const float maskHeight = static_cast<float>(nextTexture.height) * (1.0f - timeB);
        const float whiteTint[4] = { 1.0f, 1.0f, 1.0f, 1.0f };
        DrawRectangle(nextLeft, maskTop, static_cast<float>(nextTexture.width), maskHeight, whiteTint);
    }
    else if (time >= 0.25f)
    {
        const float timeB = EvaluateRevealPhaseB(time);
        const float whiteTint[4] = { 1.0f, 1.0f, 1.0f, 1.0f };
        DrawRectangle(nextLeft, nextTop, static_cast<float>(nextTexture.width), static_cast<float>(nextTexture.height) * timeB, whiteTint);
    }
}

void D3D11Renderer::CommitTransitionToBase(const Texture& previousTexture, float x, float y, float rotationDegrees, float scaleReduction)
{
    BeginRenderTarget(scratchTexture_.rtv.Get(), true);

    const float opaque[4] = { 1.0f, 1.0f, 1.0f, 1.0f };
    DrawTexture(
        baseTexture_.srv.Get(),
        static_cast<float>(screenWidth_) * 0.5f,
        static_cast<float>(screenHeight_) * 0.5f,
        static_cast<float>(screenWidth_),
        static_cast<float>(screenHeight_),
        0.0f,
        opaque);

    if (previousTexture.IsValid())
    {
        const float darkAlpha = EvaluateTimeA(1.0f) * 175.0f / 255.0f;
        const float centerX = static_cast<float>(screenWidth_) * 0.5f + x;
        const float centerY = static_cast<float>(screenHeight_) * 0.5f + y;
        const float scaledWidth = static_cast<float>(previousTexture.width) * (1.0f - scaleReduction);
        const float scaledHeight = static_cast<float>(previousTexture.height) * (1.0f - scaleReduction);
        const float blackTint[4] = { 0.0f, 0.0f, 0.0f, darkAlpha };

        DrawTexture(previousTexture.srv.Get(), centerX, centerY, scaledWidth, scaledHeight, rotationDegrees, opaque);
        DrawTexture(whiteTexture_.srv.Get(), centerX, centerY, scaledWidth, scaledHeight, rotationDegrees, blackTint);
    }

    std::swap(baseTexture_, scratchTexture_);
}

void D3D11Renderer::BeginRenderTarget(ID3D11RenderTargetView* renderTargetView, bool clear)
{
    context_->OMSetRenderTargets(1, &renderTargetView, nullptr);
    if (clear)
    {
        const float clearColor[4] = { 0.0f, 0.0f, 0.0f, 1.0f };
        context_->ClearRenderTargetView(renderTargetView, clearColor);
    }

    const float blendFactor[4] = { 0.0f, 0.0f, 0.0f, 0.0f };
    context_->OMSetBlendState(blendState_.Get(), blendFactor, 0xFFFFFFFF);
    context_->IASetInputLayout(inputLayout_.Get());
    const UINT stride = sizeof(Vertex);
    const UINT offset = 0;
    ID3D11Buffer* vertexBuffer = vertexBuffer_.Get();
    context_->IASetVertexBuffers(0, 1, &vertexBuffer, &stride, &offset);
    context_->IASetPrimitiveTopology(D3D11_PRIMITIVE_TOPOLOGY_TRIANGLELIST);
    context_->VSSetShader(vertexShader_.Get(), nullptr, 0);
    context_->PSSetShader(pixelShader_.Get(), nullptr, 0);
    ID3D11SamplerState* sampler = samplerState_.Get();
    context_->PSSetSamplers(0, 1, &sampler);
}

void D3D11Renderer::DrawTexture(
    ID3D11ShaderResourceView* shaderResourceView,
    float centerX,
    float centerY,
    float width,
    float height,
    float rotationDegrees,
    const float tint[4])
{
    if (shaderResourceView == nullptr || width <= 0.0f || height <= 0.0f)
    {
        return;
    }

    const float radians = rotationDegrees * 3.1415926535f / 180.0f;
    const float cosine = std::cos(radians);
    const float sine = std::sin(radians);
    const float halfWidth = width * 0.5f;
    const float halfHeight = height * 0.5f;

    const std::array<std::pair<float, float>, 4> corners = {
        std::pair<float, float>{ -halfWidth, -halfHeight },
        std::pair<float, float>{ halfWidth, -halfHeight },
        std::pair<float, float>{ halfWidth, halfHeight },
        std::pair<float, float>{ -halfWidth, halfHeight },
    };
    const std::array<std::pair<float, float>, 4> uvs = {
        std::pair<float, float>{ 0.0f, 0.0f },
        std::pair<float, float>{ 1.0f, 0.0f },
        std::pair<float, float>{ 1.0f, 1.0f },
        std::pair<float, float>{ 0.0f, 1.0f },
    };

    std::array<Vertex, 4> transformed = {};
    for (size_t i = 0; i < corners.size(); ++i)
    {
        const float rotatedX = corners[i].first * cosine - corners[i].second * sine;
        const float rotatedY = corners[i].first * sine + corners[i].second * cosine;
        transformed[i] = { centerX + rotatedX, centerY + rotatedY, uvs[i].first, uvs[i].second };
    }

    const std::array<Vertex, 6> vertices = {
        transformed[0], transformed[1], transformed[2],
        transformed[0], transformed[2], transformed[3],
    };

    if (!UpdateVertices(vertices.data(), vertices.size()))
    {
        return;
    }

    D3D11_MAPPED_SUBRESOURCE mappedResource = {};
    if (FAILED(context_->Map(constantBuffer_.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mappedResource)))
    {
        return;
    }

    auto* constants = static_cast<ShaderConstants*>(mappedResource.pData);
    constants->screenWidth = static_cast<float>(screenWidth_);
    constants->screenHeight = static_cast<float>(screenHeight_);
    constants->tint[0] = tint[0];
    constants->tint[1] = tint[1];
    constants->tint[2] = tint[2];
    constants->tint[3] = tint[3];
    context_->Unmap(constantBuffer_.Get(), 0);

    ID3D11Buffer* constantBuffer = constantBuffer_.Get();
    context_->VSSetConstantBuffers(0, 1, &constantBuffer);
    context_->PSSetConstantBuffers(0, 1, &constantBuffer);
    context_->PSSetShaderResources(0, 1, &shaderResourceView);
    context_->Draw(6, 0);

    ID3D11ShaderResourceView* nullShaderResourceView = nullptr;
    context_->PSSetShaderResources(0, 1, &nullShaderResourceView);
}

void D3D11Renderer::DrawRectangle(float left, float top, float width, float height, const float tint[4])
{
    DrawTexture(
        whiteTexture_.srv.Get(),
        left + width * 0.5f,
        top + height * 0.5f,
        width,
        height,
        0.0f,
        tint);
}

bool D3D11Renderer::UpdateVertices(const Vertex* vertices, size_t vertexCount)
{
    if (vertexCount != 6)
    {
        return false;
    }

    D3D11_MAPPED_SUBRESOURCE mappedResource = {};
    if (FAILED(context_->Map(vertexBuffer_.Get(), 0, D3D11_MAP_WRITE_DISCARD, 0, &mappedResource)))
    {
        return false;
    }

    std::memcpy(mappedResource.pData, vertices, sizeof(Vertex) * vertexCount);
    context_->Unmap(vertexBuffer_.Get(), 0);
    return true;
}

float D3D11Renderer::EvaluateTimeA(float time)
{
    return 1.0f - 1.2f / static_cast<float>(std::cosh(time * 3.0f * 3.1415926535f + 0.65f));
}

float D3D11Renderer::EvaluateRevealPhaseA(float time)
{
    const float timeB = time * 4.0f - 2.0f;
    return 1.0f - 1.0f / static_cast<float>(std::cosh(timeB * 2.0f * 3.1415926535f));
}

float D3D11Renderer::EvaluateRevealPhaseB(float time)
{
    const float timeB = time * 4.0f - 1.0f;
    return 1.0f / static_cast<float>(std::cosh((timeB - 1.0f) * 2.0f * 3.1415926535f));
}

bool D3D11Renderer::WaitInterruptible(int milliseconds)
{
    auto deadline = std::chrono::steady_clock::now() + std::chrono::milliseconds(milliseconds);
    while (running_.load() && std::chrono::steady_clock::now() < deadline)
    {
        if (resizePending_.exchange(false))
        {
            HandleResize();
        }

        const auto now = std::chrono::steady_clock::now();
        const auto remaining = std::chrono::duration_cast<std::chrono::milliseconds>(deadline - now).count();
        const auto sleepMs = std::min<std::int64_t>(remaining, 50);
        if (sleepMs > 0)
        {
            std::this_thread::sleep_for(std::chrono::milliseconds(sleepMs));
        }
    }

    return running_.load();
}





