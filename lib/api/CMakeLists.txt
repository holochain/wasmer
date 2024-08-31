set (WAMR_ROOT_DIR ./wasm-micro-runtime-WAMR-2.1.0)

include (${WAMR_ROOT_DIR}/build-scripts/runtime_lib.cmake)
add_library(vmlib ${WAMR_RUNTIME_LIB_SOURCE})