find_package(msgpack REQUIRED)

add_executable(tether_agent_send agent_send.cpp)
# add_executable(tether_agent_receive agent_receive.cpp)

set_property(TARGET tether_agent_send PROPERTY CXX_STANDARD 11)
# set_property(TARGET tether_agent_receive PROPERTY CXX_STANDARD 11)

target_include_directories(tether_agent_send PUBLIC ../src)
# target_include_directories(tether_agent_receive PUBLIC ../src)

target_link_libraries(tether_agent_send PUBLIC TetherAgent msgpackc-cxx)
# target_link_libraries(tether_agent_receive PUBLIC TetherAgent msgpackc-cxx)