#----------------------------------------------------------------
# Generated CMake target import file for configuration "Release".
#----------------------------------------------------------------

# Commands may need to know the format version.
set(CMAKE_IMPORT_FILE_VERSION 1)

# Import target "Serf::serf-1" for configuration "Release"
set_property(TARGET Serf::serf-1 APPEND PROPERTY IMPORTED_CONFIGURATIONS RELEASE)
set_target_properties(Serf::serf-1 PROPERTIES
  IMPORTED_IMPLIB_RELEASE "${_IMPORT_PREFIX}/lib/serf-1.lib"
  IMPORTED_LOCATION_RELEASE "${_IMPORT_PREFIX}/bin/serf-1.dll"
  )

list(APPEND _cmake_import_check_targets Serf::serf-1 )
list(APPEND _cmake_import_check_files_for_Serf::serf-1 "${_IMPORT_PREFIX}/lib/serf-1.lib" "${_IMPORT_PREFIX}/bin/serf-1.dll" )

# Commands beyond this point should not need to know the version.
set(CMAKE_IMPORT_FILE_VERSION)
