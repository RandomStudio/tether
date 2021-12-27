add_library(TetherAgent tether_agent.cpp)

set_property(TARGET TetherAgent PROPERTY CXX_STANDARD 11)

target_include_directories(TetherAgent PUBLIC ../libs/paho.mqtt.cpp/src)
target_include_directories(TetherAgent PUBLIC ../libs/paho.mqtt.c/src)
