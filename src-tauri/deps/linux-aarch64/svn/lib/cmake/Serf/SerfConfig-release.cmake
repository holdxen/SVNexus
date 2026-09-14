#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "Serf::serf-1" for configuration "Release"
set_property(TARGET Serf::serf-1 APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(Serf::serf-1 PROPERTIES
  IMPORTED_LOCATION_RELEASE "${_IMPORT_PREFIX}/lib/libserf-1.so"
  IMPORTED_SONAME_RELEASE "libserf-1.so"
  )

list(APPEND _IMPORT_CHECK_TARGETS Serf::serf-1 )
list(APPEND _IMPORT_CHECK_FILES_FOR_Serf::serf-1 "${_IMPORT_PREFIX}/lib/libserf-1.so" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
