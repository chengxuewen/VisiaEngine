# 探针消费者：find_package 同文演练 + ctest 运行锁
find_package(visiaengine REQUIRED)
add_executable(consume_capi consume_capi.cpp)
target_link_libraries(consume_capi PRIVATE visiaengine::capi)
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(consume_capi ${VISIAENGINE_CARGO_STEP}) # 排序桥（prebuilt 模式无 step）
endif()
add_test(NAME probe_abi COMMAND consume_capi)
