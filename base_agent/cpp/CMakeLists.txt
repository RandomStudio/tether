cmake_minimum_required(VERSION 3.10)
project(TetherCPP)

# Settings for MQTT libraries
SET(PAHO_BUILD_STATIC TRUE)
SET(PAHO_BUILD_WITH_SSL FALSE)

# Helpful tips for compiling CPP and C libraries together: https://github.com/eclipse/paho.mqtt.cpp/issues/225#issuecomment-586644232
include(pahomqttc.cmake)
include(pahomqttcpp.cmake)

# set(OPENSSL_ROOT_DIR /usr/local/opt/openssl@1.1)


set(PAHO_BUILD_STATIC TRUE)
# add_subdirectory(libs/paho.mqtt.c)

# set(PAHO_MQTT_C_INCLUDE_DIRS ${CMAKE_MODULE_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/paho.mqtt.c/src)
# add_subdirectory(libs/paho.mqtt.cpp)

# set(PAHO_MQTT_C_LIBRARIES ${CMAKE_CURRENT_SOURCE_DIR}/libs/paho.mqtt.c)

add_subdirectory(src)

# target_link_libraries(
#     TetherCpp
#     PRIVATE paho-mqttpp3
#     PRIVATE paho-mqttc3::MQTTAsync
# )

# set(CMAKE_PREFIX_PATH ${CMAKE_PREFIX_PATH} ${CMAKE_CURRENT_SOURCE_DIR}/libs/msgpack)

# add_subdirectory(examples)

