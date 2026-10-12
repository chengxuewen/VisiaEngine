# Probe consumer: the find_package same-text rehearsal + the ctest run lock
find_package(visiaengine REQUIRED)
add_executable(consume_capi consume_capi.cpp)
target_link_libraries(consume_capi PRIVATE visiaengine::capi)
if(DEFINED VISIAENGINE_CARGO_STEP)
  add_dependencies(consume_capi ${VISIAENGINE_CARGO_STEP}) # the ordering bridge (prebuilt mode has no step)
endif()
set_target_properties(consume_capi PROPERTIES FOLDER "bindings/c/probe")  # FOLDER = the disk-position mirror; identity semantics ride the LABELS "probe"
add_test(NAME probe_abi COMMAND consume_capi)
set_tests_properties(probe_abi PROPERTIES LABELS "probe")
