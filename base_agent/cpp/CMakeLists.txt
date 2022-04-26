cmake_minimum_required(VERSION 3.10)
project(TetherCPP)


set(OPENSSL_ROOT_DIR /usr/local/opt/openssl@1.1)


# set(PAHO_BUILD_STATIC TRUE)
# add_subdirectory(libs/paho.mqtt.c)

# set(PAHO_MQTT_C_INCLUDE_DIRS ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/paho.mqtt.c/src)
# add_subdirectory(libs/paho.mqtt.cpp)
# add_subdirectory(libs/msgpack)

set(WITH_TLS OFF)
set(WITH_STATIC_LIBRARIES ON)
set(WITH_CLIENTS OFF)
set(WITH_BROKER OFF)
set(WITH_APPS OFF)
set(WITH_PLUGINS OFF)
set(DOCUMENTATION OFF)
add_subdirectory(./libs/mosquitto)

add_subdirectory(src)
# set(CMAKE_PREFIX_PATH ${CMAKE_PREFIX_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/msgpack)

add_subdirectory(examples)

