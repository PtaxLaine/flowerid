cmake_minimum_required(VERSION 3.2)
enable_testing()

project(flowerid_cxx CXX)

list(APPEND CMAKE_MODULE_PATH "${CMAKE_SOURCE_DIR}/cmake")
find_package(cargo REQUIRED)
find_package(rustc REQUIRED)

cargo_build_library(
	SOURCE_DIR ${CMAKE_SOURCE_DIR}/../flowerid_c
	TARGET_MODE release
	TARGET_TYPE staticlib
	RESULT flowerid_c_lib)

include_directories("${CMAKE_SOURCE_DIR}/include/")
include_directories("${CMAKE_SOURCE_DIR}/googletest/googletest/include/")
include_directories("${CMAKE_SOURCE_DIR}/googletest/googletest/")
configure_file("${CMAKE_SOURCE_DIR}/../flowerid_c/include/flowerid.h" "${CMAKE_SOURCE_DIR}/include/flowerid_ex.h" COPYONLY)

add_library(flowerid_cxx flowerid.cpp)
add_executable(flowerid_cxx_test test.cpp "${CMAKE_SOURCE_DIR}/googletest/googletest/src/gtest_main.cc" "${CMAKE_SOURCE_DIR}/googletest/googletest/src/gtest-all.cc")
add_dependencies(flowerid_cxx_test flowerid_cxx)
#add_custom_command(TARGET flowerid_cxx_test POST_BUILD COMMAND ctest)

add_test( Test1 flowerid_cxx_test )

target_link_libraries(flowerid_cxx ${flowerid_c_lib})
target_link_libraries(flowerid_cxx_test flowerid_cxx)
if(CMAKE_SYSTEM_NAME STREQUAL "Windows")
	target_link_libraries(flowerid_cxx Ws2_32 Userenv)
else()
	target_link_libraries(flowerid_cxx pthread dl)
endif()

if(MSVC)
  # Force to always compile with W4
  if(CMAKE_CXX_FLAGS MATCHES "/W[0-4]")
    string(REGEX REPLACE "/W[0-4]" "/W4" CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS}")
  else()
    set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} /W4")
  endif()
elseif(CMAKE_COMPILER_IS_GNUCC OR CMAKE_COMPILER_IS_GNUCXX)
  # Update if necessary
  set(CMAKE_CXX_FLAGS "${CMAKE_CXX_FLAGS} -Wall -Wno-long-long -pedantic")
endif()
