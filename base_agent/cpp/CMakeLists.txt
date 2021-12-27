cmake_minimum_required(VERSION 3.10)

set(OPENSSL_ROOT_DIR /usr/local/opt/openssl@1.1)
set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/msgpack)
project(TetherCPP)

add_subdirectory(libs/paho.mqtt.c)
add_subdirectory(libs/paho.mqtt.cpp)
add_subdirectory(libs/msgpack)
add_subdirectory(src)
add_subdirectory(examples)

