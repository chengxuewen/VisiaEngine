# Install tree (packaging-round v1.3 P2).
# Design premise, measured rather than assumed: the cdylib is produced by cargo,
# so CMake never *owns* it. Two consequences follow:
#   * install(TARGETS <imported>)/install(EXPORT) is not available at all --
#     CMake rejects it with "EXPORT_NAME property can't be set on imported
#     targets" and "install TARGETS given target ... which does not exist";
#   * the installed package config therefore creates the imported target itself
#     from a template, deriving every path from PACKAGE_PREFIX_DIR (computed
#     from the config file's own location), which is what makes the tree
#     relocatable (verified: rebuild a consumer against a moved prefix).
# Header names and positions are kept byte-identical to the in-tree layout so
# that "the same consumer source compiles inside the repo and after install"
# holds for all three include spellings in use today ("visiaengine.h",
# <visiaengine/visiaengine.hpp>, "visiaengine_widget.hpp").

function(visiaengine_install_tree)
  include(GNUInstallDirs)
  include(CMakePackageConfigHelpers)

  set(_art "${VISIAENGINE_ARTIFACT}")
  if(NOT EXISTS "${_art}")
    message(FATAL_ERROR
      "visiaengine: asked to install but the artifact is absent: ${_art}\n"
      "  build tree : configure+build first (cargo-build_capi runs automatically)\n"
      "  prebuilt   : -DVISIAENGINE_ARTIFACT_PATH=<dir containing libvisiaengine.so>")
  endif()

  # Library
  install(FILES "${_art}" DESTINATION "${CMAKE_INSTALL_LIBDIR}")

  # Public headers: C face at include/ top level, C++ face in include/visiaengine/,
  # Qt widget header beside the C one (it includes "visiaengine.h" by bare name).
  install(FILES
      "${PROJECT_SOURCE_DIR}/bindings/c/visiaengine-capi/include/visiaengine.h"
      "${PROJECT_SOURCE_DIR}/bindings/qt/visiaengine_widget.hpp"
    DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}")
  install(FILES
      "${PROJECT_SOURCE_DIR}/bindings/cpp/include/visiaengine/visiaengine.hpp"
    DESTINATION "${CMAKE_INSTALL_INCLUDEDIR}/visiaengine")

  # License texts ship with the SDK (dual-licensed core, non-revocable promise).
  install(FILES
      "${PROJECT_SOURCE_DIR}/LICENSE-MIT"
      "${PROJECT_SOURCE_DIR}/LICENSE-APACHE"
    DESTINATION "${CMAKE_INSTALL_DATADIR}/licenses/${PROJECT_NAME}")

  # Package config (+ version). Generated at configure time so CPack/the install
  # manifest pick them up without an extra build step. SameMajorVersion: the
  # runtime ABI number (visiaengine_abi_version) is the real compatibility
  # contract and is pinned by the consumer probes, not by this file.
  configure_package_config_file(
    "${PROJECT_SOURCE_DIR}/cmake/visiaengineConfig.cmake.in"
    "${CMAKE_CURRENT_BINARY_DIR}/visiaengineConfig.cmake"
    INSTALL_DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")
  write_basic_package_version_file(
    "${CMAKE_CURRENT_BINARY_DIR}/visiaengineConfigVersion.cmake"
    VERSION "${PROJECT_VERSION}"
    COMPATIBILITY SameMajorVersion)
  install(FILES
      "${CMAKE_CURRENT_BINARY_DIR}/visiaengineConfig.cmake"
      "${CMAKE_CURRENT_BINARY_DIR}/visiaengineConfigVersion.cmake"
    DESTINATION "${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME}")

  message(STATUS "visiaengine install tree: ${CMAKE_INSTALL_PREFIX}/${CMAKE_INSTALL_LIBDIR}"
                 " + ${CMAKE_INSTALL_LIBDIR}/cmake/${PROJECT_NAME} (find_package name: ${PROJECT_NAME})")
endfunction()
