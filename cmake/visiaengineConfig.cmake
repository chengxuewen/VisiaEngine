# 树内假 Config（消费代码树内/装后同文；install 真身=打包轮，D-11 复评在册）
if(NOT TARGET visiaengine::capi)
  message(FATAL_ERROR
    "visiaengine 当前唯一消费路径=仓内 add_subdirectory（根 CMakeLists）；install 树=打包轮交付")
endif()
set(visiaengine_FOUND TRUE)
