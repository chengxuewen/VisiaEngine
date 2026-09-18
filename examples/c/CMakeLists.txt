# examples/c —— C 例真身（Easy3D 对标带的 C 语料位；R10 命名法：真身=E 号原名）
# 反-glob 中央账在 visiaengine_setup_bindings()（盘上有件未注册=configure 硬错）
find_package(X11 QUIET)
# SDL3=窗口例宿主（S0 实测通路）；conda 件发现=激活壳 CMAKE_PREFIX_PATH 优先，仓内兜底
if(NOT DEFINED CMAKE_PREFIX_PATH OR CMAKE_PREFIX_PATH STREQUAL "")
  list(APPEND CMAKE_PREFIX_PATH "${CMAKE_SOURCE_DIR}/.pixi/envs/default")
endif()
find_package(SDL3 QUIET CONFIG)

visiaengine_add_example(E701_demo_headless)
visiaengine_add_example(E810_attr_survey)   # 数据带：属性遍历 survey（零新口，CAPI-04/10 全景）
visiaengine_add_example(E811_entity_hide_c)
visiaengine_add_example(E813_section_clip)
visiaengine_add_example(E814_labels_headless)
visiaengine_add_example(E815_fly_camera)  # 飞行带：墙钟推进/进度单调/cancel（CAPI-23/24 活体门）  # 文字带：字体注入+锚标签+utf8 活体门（CAPI-21/22）  # 剖面带：set/get_clips+拾取重试+值域拒（CAPI-20 活体门）  # 数据带：显隐孪生·C 面（CAPI-13/14）
visiaengine_add_example(E704_host_callback)  # 事件带：CAPI-17 推送口三态验收（进度/错误/摘除）
visiaengine_add_example(E816_minimap_nav)  # 小地图带：canary 逐字节/角区族/小图 pick/导航到站/值域拒（CAPI-25..27 活体门）
if(SDL3_FOUND)
  visiaengine_add_example(E801_sdl_window DISPLAY SDL3 ARGS "--frames;3")  # 双模：无参交互窗（run_E801 路）/--frames headless 快退（ctest 路）
else()
  set_property(GLOBAL APPEND PROPERTY VE_REGISTERED_EXAMPLES E801_sdl_window)
  message(STATUS "examples/c：SDL3 缺位，E801 跳注册（pixi tools 含 sdl3 时点亮）")
endif()
if(X11_FOUND)
  visiaengine_add_example(E702_demo_x11 DISPLAY X11 ARGS "--frames;3")  # 双模（E702 修案：原固定 3 帧=约定漏网）
else()
  # 缺件优雅降级（双层门控旧形）：中央账免死——登记名字但不建目标
  set_property(GLOBAL APPEND PROPERTY VE_REGISTERED_EXAMPLES E702_demo_x11)
  message(STATUS "examples/c：X11 缺位，E702 跳注册（pixi tools 含 xorg-libx11 时点亮）")
endif()
# S4 起本目录增生：E801_sdl_window（SDL3 窗口带首婴）+ template.c 骨架
