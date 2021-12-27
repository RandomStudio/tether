cmake_minimum_required(VERSION 3.10)

set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/msgpack)
project(TetherCPP)

add_subdirectory(libs/paho.mqtt.cpp)
add_subdirectory(libs/msgpack)
add_subdirectory(examples)

