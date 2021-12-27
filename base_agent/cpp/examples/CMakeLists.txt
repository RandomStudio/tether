find_package(msgpack REQUIRED)

add_executable(publish_example publish.cpp)
add_executable(msgpack_publish_example msgpack.cpp)

set_property(TARGET publish_example PROPERTY CXX_STANDARD 11)
set_property(TARGET msgpack_publish_example PROPERTY CXX_STANDARD 11)

target_include_directories(publish_example PUBLIC ../libs/paho.mqtt.cpp/src)
target_include_directories(publish_example PUBLIC ../libs/paho.mqtt.c/src)

target_include_directories(msgpack_publish_example PUBLIC ../libs/paho.mqtt.cpp/src)
target_include_directories(msgpack_publish_example PUBLIC ../libs/paho.mqtt.c/src)

target_link_libraries(publish_example PUBLIC paho-mqttpp3)
target_link_libraries(msgpack_publish_example PUBLIC paho-mqttpp3 msgpackc-cxx)