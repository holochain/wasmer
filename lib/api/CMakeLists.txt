cmake_minimum_required (VERSION 3.0)

project (iwasm)

set (WAMR_ROOT_DIR ./third_party/wamr)

include (${WAMR_ROOT_DIR}/build-scripts/runtime_lib.cmake)
add_library(vmlib ${WAMR_RUNTIME_LIB_SOURCE})