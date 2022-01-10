cmake_minimum_required(VERSION 3.10)
project(TetherCPP)

set(OPENSSL_ROOT_DIR /usr/local/opt/openssl@1.1)
set(CMAKE_MODULE_PATH ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/msgpack)

set(PAHO_BUILD_STATIC TRUE)
add_subdirectory(libs/paho.mqtt.c)

set(PAHO_MQTT_C_INCLUDE_DIRS ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/paho.mqtt.c/src)
add_subdirectory(libs/paho.mqtt.cpp)
add_subdirectory(libs/msgpack)
add_subdirectory(src)
add_subdirectory(examples)

