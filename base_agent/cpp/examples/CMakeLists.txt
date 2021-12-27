find_package(msgpack REQUIRED)

add_executable(publish_example publish.cpp)
add_executable(msgpack_publish_example msgpack.cpp)
add_executable(tether_agent_example agent.cpp)

set_property(TARGET publish_example PROPERTY CXX_STANDARD 11)
set_property(TARGET msgpack_publish_example PROPERTY CXX_STANDARD 11)
set_property(TARGET tether_agent_example PROPERTY CXX_STANDARD 11)

target_include_directories(publish_example PUBLIC ../libs/paho.mqtt.cpp/src)
target_include_directories(publish_example PUBLIC ../libs/paho.mqtt.c/src)

target_include_directories(msgpack_publish_example PUBLIC ../libs/paho.mqtt.cpp/src)
target_include_directories(msgpack_publish_example PUBLIC ../libs/paho.mqtt.c/src)

target_include_directories(tether_agent_example PUBLIC ../src)

target_link_libraries(publish_example PUBLIC paho-mqttpp3)
target_link_libraries(msgpack_publish_example PUBLIC paho-mqttpp3 msgpackc-cxx)
target_link_libraries(tether_agent_example PUBLIC TetherAgent paho-mqttpp3 msgpackc-cxx)