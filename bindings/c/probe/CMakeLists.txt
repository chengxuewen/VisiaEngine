# 探针消费者：find_package 同文演练 + ctest 运行锁
find_package(visiaengine REQUIRED)
add_executable(consume_capi consume_capi.cpp)
target_link_libraries(consume_capi PRIVATE visiaengine::capi)
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(consume_capi ${VISIAENGINE_CARGO_STEP}) # 排序桥（prebuilt 模式无 step）
endif()
set_target_properties(consume_capi PROPERTIES FOLDER "bindings/c/probe")  # FOLDER=磁盘位置镜像；身份语义走 LABELS "probe"
add_test(NAME probe_abi COMMAND consume_capi)
set_tests_properties(probe_abi PROPERTIES LABELS "probe")
