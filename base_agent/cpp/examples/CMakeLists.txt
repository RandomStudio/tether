cmake_minimum_required(VERSION 3.19)
find_package(msgpack REQUIRED)

project(TetherExample)


add_executable(tether-agent-example ./agent.cpp)
set_property(TARGET tether-agent-example PROPERTY CXX_STANDARD 17)
target_include_directories(tether-agent-example PUBLIC ../src)
# target_include_directories(tether-agent-example PUBLIC ../libs/msgpack/include)

target_link_libraries(tether-agent-example PUBLIC TetherAgent msgpackc-cxx)
