# cargo-step + 伞合同 visiaengine::capi + 树内假 Config 桥（§2.3；iceoryx2 形，
# 设计=.omo/plans/visiaengine-cmake-project.md v1.0）

function(visiaengine_setup_cargo)
  set(_src ${CMAKE_SOURCE_DIR}/crates/visiaengine-capi)
  if(VISIAENGINE_ARTIFACT_PATH)
    set(_dir "${VISIAENGINE_ARTIFACT_PATH}")
    set(_flags "")
  else()
    set(_dir "${CMAKE_BINARY_DIR}/rust")
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
      set(_profile "release")
      set(_flags "--release")
    else()
      set(_profile "debug")
      set(_flags "")
    endif()
    set(_dir "${CMAKE_BINARY_DIR}/rust/${_profile}")
  endif()

  # 逐 OS 产物名（仅命名表六行，无安装语义）
  if(WIN32)
    set(_shared "visiaengine.dll")
    set(_static "visiaengine.lib")
  elseif(APPLE)
    set(_shared "libvisiaengine.dylib")
    set(_static "libvisiaengine.a")
  else()
    set(_shared "libvisiaengine.so")
    set(_static "libvisiaengine.a")
  endif()

  if(NOT VISIAENGINE_ARTIFACT_PATH)
    add_custom_target(visiaengine-cargo-step ALL
      COMMAND ${VISIAENGINE_CARGO} build -p visiaengine-capi ${_flags}
              --target-dir ${CMAKE_BINARY_DIR}/rust
      BYPRODUCTS ${_dir}/${_shared} ${_dir}/${_static}
      WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}
      VERBATIM USES_TERMINAL)
    set(VISIAENGINE_CARGO_STEP "visiaengine-cargo-step" PARENT_SCOPE)
  endif()

  add_library(visiaengine_capi_umbrella INTERFACE)
  add_library(visiaengine::capi ALIAS visiaengine_capi_umbrella)
  target_include_directories(visiaengine_capi_umbrella INTERFACE
    "$<BUILD_INTERFACE:${_src}/include>")
  target_link_libraries(visiaengine_capi_umbrella INTERFACE "${_dir}/${_shared}")
  if(UNIX AND NOT APPLE) # 树内消费免 LD_LIBRARY_PATH（前作手动 env 之债不抄）
    target_link_options(visiaengine_capi_umbrella INTERFACE "-Wl,-rpath,${_dir}")
  endif()

  # 树内假 Config（iceoryx2 双态花招，install 生成版=打包轮）：
  # 子目录 find_package(visiaengine) 命中本文件 → 目标已在树内定义即成功
  set(visiaengine_DIR "${CMAKE_SOURCE_DIR}/cmake" CACHE PATH "visiaengine in-tree config" FORCE)
  message(STATUS "visiaengine::capi 伞就位：${_dir}/${_shared}")
endfunction()
