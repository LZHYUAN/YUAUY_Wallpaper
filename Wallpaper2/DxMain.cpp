# pragma once
#include <windows.h>
#include <d3d11.h>
#include <dxgi1_2.h>
#include <d3dcompiler.h>
#include <DirectXMath.h>

#pragma comment(lib, "d3d11.lib")
#pragma comment(lib, "d3dcompiler.lib")

class DxMain
{
public:

	ID3D11Device* pDevice = nullptr;
	ID3D11DeviceContext* pDeviceContext = nullptr;

	IDXGIDevice* pDXGIDevice = nullptr;
	IDXGIAdapter* pDXGIAdapter = nullptr;
	IDXGIFactory2* pIDXGIFactory = nullptr;
	IDXGISwapChain1* pSwapChain = nullptr;


	HRESULT CreateD3D(HWND hWnd)
	{
		HRESULT hr = S_OK;

		/* D3D11CreateDevice */

		D3D_FEATURE_LEVEL FeatureLevelsRequested = D3D_FEATURE_LEVEL_11_1;
		UINT numFeatureLevelsRequested = 1;
		D3D_FEATURE_LEVEL FeatureLevelsSupported;
		hr = D3D11CreateDevice(
			NULL, // pAdapter (null for default)
			D3D_DRIVER_TYPE::D3D_DRIVER_TYPE_HARDWARE, // DriverType
			NULL, // Software
			D3D11_CREATE_DEVICE_FLAG::D3D11_CREATE_DEVICE_DEBUG, // Flags
			&FeatureLevelsRequested, // pFeatureLevels
			numFeatureLevelsRequested, // FeatureLevels
			D3D11_SDK_VERSION, // SDKVersion
			&pDevice, // [out] ppDevice
			&FeatureLevelsSupported, // [out] pFeatureLevel
			&pDeviceContext // [out] ppImmediateContex
		);
		if (FAILED(hr))
			return hr;
		if (FeatureLevelsSupported != FeatureLevelsRequested)
			return E_SUBPROTOCOL_NOT_SUPPORTED;


		/* Query IDXGIFactory */

		hr = pDevice->QueryInterface(__uuidof(IDXGIDevice), (void**)&pDXGIDevice);
		if (FAILED(hr)) return hr;
		hr = pDXGIDevice->GetAdapter(&pDXGIAdapter);
		if (FAILED(hr)) return hr;
		hr = pDXGIAdapter->GetParent(__uuidof(IDXGIFactory2), (void**)&pIDXGIFactory);
		if (FAILED(hr)) return hr;


		/* CreateSwapChainForHwnd */

		DXGI_SWAP_CHAIN_DESC1 sd = {
			0, // Width (hwnd for auto)
			0, // Height (hwnd for auto)
			DXGI_FORMAT::DXGI_FORMAT_R8G8B8A8_UNORM, // Format
			FALSE, // Stereo
			{.Count = 1,.Quality = 0}, // SampleDesc
			DXGI_USAGE_RENDER_TARGET_OUTPUT, // BufferUsage
			2, // BufferCount
			DXGI_SCALING::DXGI_SCALING_NONE, // Scaling
			DXGI_SWAP_EFFECT::DXGI_SWAP_EFFECT_FLIP_DISCARD , // SwapEffect
			 DXGI_ALPHA_MODE::DXGI_ALPHA_MODE_IGNORE, // AlphaMode
			NULL //DXGI_SWAP_CHAIN_FLAG::DXGI_SWAP_CHAIN_FLAG_FOREGROUND_LAYER // Flags
		};
		hr = pIDXGIFactory->CreateSwapChainForHwnd(
			pDevice, // pDevice (IUnknown*)
			hWnd, // hWnd
			&sd, // pDesc
			NULL, // pFullscreenDesc
			NULL, // pRestrictToOutput
			&pSwapChain // [out] ppSwapChain
		);

		if (FAILED(hr)) return hr;


		return S_OK;
	}

	ID3D11RenderTargetView* pRenderTargetView = nullptr;

	HRESULT SetViewports(UINT width, UINT height)
	{
		HRESULT hr = S_OK;

		// Release old render target view
		if (pRenderTargetView != nullptr)
			pRenderTargetView->Release();

		// Update buffer
		ID3D11Texture2D* pBackBuffer = nullptr;
		pSwapChain->ResizeBuffers(0, width, height, DXGI_FORMAT_UNKNOWN, NULL);
		hr = pSwapChain->GetBuffer(0, __uuidof(ID3D11Texture2D), (LPVOID*)&pBackBuffer);
		if (FAILED(hr)) return hr;

		// Create render target view
		hr = pDevice->CreateRenderTargetView(pBackBuffer, NULL, &pRenderTargetView);
		pBackBuffer->Release();
		if (FAILED(hr)) return hr;

		pDeviceContext->OMSetRenderTargets(1, &pRenderTargetView, NULL);

		// Setup the viewport
		D3D11_VIEWPORT vp = {
			.TopLeftX = 0,
			.TopLeftY = 0,
			.Width = (FLOAT)width,
			.Height = (FLOAT)height,
			.MinDepth = 0.0f,
			.MaxDepth = 1.0f,
		};
		pDeviceContext->RSSetViewports(1, &vp);

		return S_OK;
	}

	void Render()
	{
		float clearColor[4] = { 0.1f, 0.2f, 0.3f, 1.0f }; // RGBA

		pDeviceContext->ClearRenderTargetView(pRenderTargetView, clearColor);
		pDeviceContext->
		pSwapChain->Present(0, 0);
	}

};
