
add_library(TetherAgent tether_agent.cpp)

set_property(TARGET TetherAgent PROPERTY CXX_STANDARD 11)

# find_package(PkgConfig REQUIRED)
# pkg_check_modules(Mosquitto REQUIRED IMPORTED_TARGET mosquitto)


target_include_directories(TetherAgent PUBLIC ../libs/mosquitto/include)
# add_subdirectory(../libs/mosquitto)
target_link_libraries(TetherAgent PUBLIC mosquittopp_static libmosquitto_static)