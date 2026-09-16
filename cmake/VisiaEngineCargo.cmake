# cargo-step + 伞合同 visiaengine::capi + 树内假 Config 桥（§2.3；iceoryx2 形，
# 设计=.omo/plans/visiaengine-cmake-project.md v1.0）

function(visiaengine_setup_cargo)
  set(_src ${CMAKE_SOURCE_DIR}/bindings/c/visiaengine-capi)
  if(VISIAENGINE_ARTIFACT_PATH)
    set(_dir "${VISIAENGINE_ARTIFACT_PATH}")
    set(_flags "")
  else()
    set(_rust_dir "${CMAKE_SOURCE_DIR}/target/cmake-rust") # 仓级共享 target-dir：
    # 多构建目录复用同一 cargo 增量面（cargo 按 profile 子目录隔离 debug/release，锁串行安全）
    if(CMAKE_BUILD_TYPE STREQUAL "Release")
      set(_profile "release")
      set(_flags "--release")
    else()
      set(_profile "debug")
      set(_flags "")
    endif()
    set(_dir "${_rust_dir}/${_profile}")
    # 暴露给 examples 层（VisiaEngineExamples.cmake 复用同一产物目录合同）
    set(VISIAENGINE_RUST_DIR "${_rust_dir}" PARENT_SCOPE)
    set(VISIAENGINE_RUST_PROFILE "${_profile}" PARENT_SCOPE)
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
    # IDE 直调守卫：conda cargo 按 PATH 找 rustc，裸 PATH 下=「could not execute rustc」
    # （2026-09-15 实锤：pixi run 包装验证掩盖，用户 IDE /usr/bin/cmake 直调首爆）——
    # 构建命令恒定前缀 cargo 同目录（rustc 兄弟件实测在场），SYSTEM 态无害。
    get_filename_component(_cargo_bin "${VISIAENGINE_CARGO}" DIRECTORY)
    add_custom_target(visiaengine-cargo-step ALL
      COMMAND ${CMAKE_COMMAND} -E env "PATH=${_cargo_bin}:$ENV{PATH}"
              ${VISIAENGINE_CARGO} build -p visiaengine-capi ${_flags}
              --target-dir ${_rust_dir}
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
