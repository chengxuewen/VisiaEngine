# examples/c —— C 例真身（Easy3D 对标带的 C 语料位；R10 命名法：真身=E 号原名）
# 反-glob 中央账在 visiaengine_setup_bindings()（盘上有件未注册=configure 硬错）
find_package(X11 QUIET)
# SDL3=窗口例宿主（S0 实测通路）；conda 件发现=激活壳 CMAKE_PREFIX_PATH 优先，仓内兜底
if(NOT DEFINED CMAKE_PREFIX_PATH OR CMAKE_PREFIX_PATH STREQUAL "")
  list(APPEND CMAKE_PREFIX_PATH "${CMAKE_SOURCE_DIR}/.pixi/envs/default")
endif()
find_package(SDL3 QUIET CONFIG)

visiaengine_add_example(E701_demo_headless)
if(SDL3_FOUND)
  visiaengine_add_example(E801_sdl_window DISPLAY SDL3)
else()
  set_property(GLOBAL APPEND PROPERTY VE_REGISTERED_EXAMPLES E801_sdl_window)
  message(STATUS "examples/c：SDL3 缺位，E801 跳注册（pixi tools 含 sdl3 时点亮）")
endif()
if(X11_FOUND)
  visiaengine_add_example(E702_demo_x11 DISPLAY X11)
else()
  # 缺件优雅降级（双层门控旧形）：中央账免死——登记名字但不建目标
  set_property(GLOBAL APPEND PROPERTY VE_REGISTERED_EXAMPLES E702_demo_x11)
  message(STATUS "examples/c：X11 缺位，E702 跳注册（pixi tools 含 xorg-libx11 时点亮）")
endif()
# S4 起本目录增生：E801_sdl_window（SDL3 窗口带首婴）+ template.c 骨架
