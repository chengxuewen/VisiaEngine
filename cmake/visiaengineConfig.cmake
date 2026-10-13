# In-tree fake Config (in-tree consumers / post-install consumers read the same text; the install real one = packaging round, D-11 re-review on the books)
if(NOT TARGET visiaengine::c)
  message(FATAL_ERROR
    "visiaengine's only consumption path today = the in-repo add_subdirectory (root CMakeLists); the install tree = packaging-round delivery")
endif()
set(visiaengine_FOUND TRUE)
