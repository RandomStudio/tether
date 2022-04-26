
add_library(TetherAgent tether_agent.cpp)

set_property(TARGET TetherAgent PROPERTY CXX_STANDARD 11)

target_include_directories(TetherAgent PUBLIC ../libs/mosquitto/include)
# add_subdirectory(../libs/mosquitto)
target_link_libraries(TetherAgent PUBLIC mosquitto)