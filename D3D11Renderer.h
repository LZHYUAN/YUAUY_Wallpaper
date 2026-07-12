#pragma once

#ifndef NOMINMAX
#define NOMINMAX
#endif

#include <d3d11.h>
#include <wincodec.h>
#include <windows.h>
#include <wrl/client.h>

#include <atomic>
#include <cstdint>
#include <filesystem>
#include <random>
#include <thread>
#include <vector>

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
    struct Texture
    {
        Microsoft::WRL::ComPtr<ID3D11Texture2D> texture;
        Microsoft::WRL::ComPtr<ID3D11ShaderResourceView> srv;
        int width = 0;
        int height = 0;

        bool IsValid() const;
    };

    struct RenderTexture
    {
        Microsoft::WRL::ComPtr<ID3D11Texture2D> texture;
        Microsoft::WRL::ComPtr<ID3D11RenderTargetView> rtv;
        Microsoft::WRL::ComPtr<ID3D11ShaderResourceView> srv;
        int width = 0;
        int height = 0;

        bool IsValid() const;
    };

    struct Vertex
    {
        float x;
        float y;
        float u;
        float v;
    };

    struct ShaderConstants
    {
        float screenWidth;
        float screenHeight;
        float padding[2];
        float tint[4];
    };

    void RenderLoop();
    bool CreateBackBufferTarget();
    bool CreateDeviceResources();
    bool CreateSceneResources();
    bool CreateRenderTexture(int width, int height, RenderTexture& target);
    bool CreateTextureFromFile(const std::filesystem::path& path, Texture& texture);
    bool CreateTextureFromPixels(const std::uint8_t* pixels, int width, int height, Texture& texture);
    bool CreateWhiteTexture();
    bool CreateWicFactory();
    void ReleaseBackBufferTarget();
    void Cleanup();
    void HandleResize();
    void PerformStartupFade();
    void RunTransitionLoop();
    bool LoadNextTexture(Texture& texture);
    void CollectImagePaths();
    void ShuffleImagePaths();
    void RenderStaticScene();
    void RenderTransitionFrame(const Texture& nextTexture, float time);
    void CommitTransitionToBase(const Texture& previousTexture, float x, float y, float rotationDegrees, float scaleReduction);
    void BeginRenderTarget(ID3D11RenderTargetView* renderTargetView, bool clear);
    void DrawTexture(
        ID3D11ShaderResourceView* shaderResourceView,
        float centerX,
        float centerY,
        float width,
        float height,
        float rotationDegrees,
        const float tint[4]);
    void DrawRectangle(
        float left,
        float top,
        float width,
        float height,
        const float tint[4]);
    bool UpdateVertices(const Vertex* vertices, size_t vertexCount);
    static float EvaluateTimeA(float time);
    static float EvaluateRevealPhaseA(float time);
    static float EvaluateRevealPhaseB(float time);
    bool WaitInterruptible(int milliseconds);

    HWND window_ = nullptr;
    Microsoft::WRL::ComPtr<ID3D11Device> device_;
    Microsoft::WRL::ComPtr<ID3D11DeviceContext> context_;
    Microsoft::WRL::ComPtr<IDXGISwapChain> swapChain_;
    Microsoft::WRL::ComPtr<ID3D11RenderTargetView> backBufferRtv_;
    Microsoft::WRL::ComPtr<ID3D11VertexShader> vertexShader_;
    Microsoft::WRL::ComPtr<ID3D11PixelShader> pixelShader_;
    Microsoft::WRL::ComPtr<ID3D11InputLayout> inputLayout_;
    Microsoft::WRL::ComPtr<ID3D11Buffer> vertexBuffer_;
    Microsoft::WRL::ComPtr<ID3D11Buffer> constantBuffer_;
    Microsoft::WRL::ComPtr<ID3D11SamplerState> samplerState_;
    Microsoft::WRL::ComPtr<ID3D11BlendState> blendState_;
    Microsoft::WRL::ComPtr<IWICImagingFactory> wicFactory_;
    Texture whiteTexture_;
    Texture lastTexture_;
    RenderTexture baseTexture_;
    RenderTexture scratchTexture_;
    std::vector<std::filesystem::path> imagePaths_;
    size_t imageCursor_ = 0;
    std::mt19937 randomEngine_;
    std::thread renderThread_;
    std::atomic<bool> running_ = false;
    std::atomic<bool> resizePending_ = false;
    bool first_ = true;
    int firstCount_ = 0;
    int screenWidth_ = 0;
    int screenHeight_ = 0;
    float transitionX_ = 0.0f;
    float transitionY_ = 0.0f;
    float transitionRotationDegrees_ = 0.0f;
};
