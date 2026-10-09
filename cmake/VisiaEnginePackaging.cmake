# CPack assembly (publish band, plan next-band-queue todo 3).
#
# Scope discipline: this module adds NO install() rules. CPack's TGZ generator
# packages the install manifest that cmake/VisiaEngineInstall.cmake already
# produces, so the tarball equals `cmake --install` file-for-file (witness:
# .omo/evidence/todo3-tarball-list.txt on this band; listing taken from the
# built archive). The artifact is a LOCAL tarball -- nothing here publishes,
# uploads, or deploys anything.
#
# RUNPATH disposition -- DECISION (restates the D23 P3 ruling; adds no behavior):
#   * The known RUNPATH leak on this machine is traced to this env's relocated
#     conda .pc files naming a sibling checkout's prefix, feeding a -sys link
#     line. Adjudicated as a publish-pipeline ASSEMBLY concern, not a repo
#     defect.
#   * A publish pipeline assembled from clean prefixes yields the ordinary
#     story: system libraries in NEEDED only, RUNPATH absent or $ORIGIN-relative.
#   * Deliberately NO patchelf and no other new tool dependency.
#     scripts/gate-pack.sh reports RUNPATH without judging it (header note);
#     the packaged library's `readelf -d` NEEDED/SONAME/RUNPATH lines are
#     recorded verbatim in the evidence file.
#
# Invocation (why cpack directly, not --target package): CPack wires `package`
# as a dependent of the build-system `install` target, which would write into
# CMAKE_INSTALL_PREFIX; running cpack on the configured dir stages the install
# itself under the build tree (DESTDIR semantics), so a non-root machine needs
# no writable prefix:
#   pixi run cmake --preset bare && pixi run cmake --build --preset bare
#   .pixi/envs/default/bin/cpack --config target/cmake-bare/CPackConfig.cmake \
#       -B target/cmake-bare
# CPack's staging install runs the same cmake_install.cmake as `cmake --install`,
# so the packaged content is the install-tree content by construction.

set(CPACK_PACKAGE_NAME "${PROJECT_NAME}")
set(CPACK_PACKAGE_VERSION "${PROJECT_VERSION}") # single source: workspace Cargo.toml (facade version read)
set(CPACK_PACKAGE_DESCRIPTION_SUMMARY
    "VisiaEngine spatial visualization SDK: cdylib + C/C++/Qt headers + package configs")
set(CPACK_PACKAGE_VENDOR "VisiaEngine")
# CPack wants exactly one license file; both dual-license texts ship inside the
# tree at share/licenses/ and the non-revocable promise lives in README.
set(CPACK_RESOURCE_FILE_LICENSE "${PROJECT_SOURCE_DIR}/LICENSE-MIT")
set(CPACK_GENERATOR "TGZ")
# Deterministic local name (default appends host sysname/arch noise).
set(CPACK_PACKAGE_FILE_NAME "visiaengine-sdk-${PROJECT_VERSION}")
include(CPack)
