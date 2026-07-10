# CMakeList.txt: YUAUY_Wallpaper_DX 的 CMake 專案，在此包含來源及定義
# 專案專屬邏輯。
#
cmake_minimum_required (VERSION 3.10)

# 如果支援，則為 MSVC 編譯器啟用熱重新載入。
if (POLICY CMP0141)
  cmake_policy(SET CMP0141 NEW)
  set(CMAKE_MSVC_DEBUG_INFORMATION_FORMAT "$<IF:$<AND:$<C_COMPILER_ID:MSVC>,$<CXX_COMPILER_ID:MSVC>>,$<$<CONFIG:Debug,RelWithDebInfo>:EditAndContinue>,$<$<CONFIG:Debug,RelWithDebInfo>:ProgramDatabase>>")
endif()

project ("YUAUY_Wallpaper_DX")

# 將來源新增到此專案的可執行檔。
add_executable (YUAUY_Wallpaper_DX WIN32 "YUAUY_Wallpaper_DX.cpp" "YUAUY_Wallpaper_DX.h")
target_link_libraries(YUAUY_Wallpaper_DX PRIVATE user32 gdi32)
target_compile_definitions(YUAUY_Wallpaper_DX PRIVATE UNICODE _UNICODE)

if (CMAKE_VERSION VERSION_GREATER 3.12)
  set_property(TARGET YUAUY_Wallpaper_DX PROPERTY CXX_STANDARD 20)
endif()

# TODO: 新增測試，並視需要安裝目標。
