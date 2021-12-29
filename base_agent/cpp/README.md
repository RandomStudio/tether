## Warning

This library currently does Output Plugs only, i.e. it can publish messages but cannot subscribe/consume!

## TODO

- [x] Demo: Basic MQTT publish working
- [x] Demo: basic MessagePack serialisation
- [x] Demo: Get basic MessagePack serialised message into MQTT body
- [x] Build actual library that can be included from other CPP projects, rather than sitting inside `examples` folder - using CMake again
- [x] Merge this project into the monorepo
- [x] Test out using CMake to import (and/or install?) this library automatically, and document the steps needed for developers to do the same
- [ ] Get receiving (subscribing) working!

## Third-party libraries used

- MQTT: https://github.com/eclipse/paho.mqtt.cpp
  - which depends on https://github.com/eclipse/paho.mqtt.c.git
- MessagePack: https://github.com/msgpack/msgpack-c/tree/cpp_master
  - which depends on Boost

## Resources

Useful guides for CMake structure

- https://hsf-training.github.io/hsf-training-cmake-webpage/06-projectstructure/index.html

## Build process:

Be sure to do
`git submodule update --init --recursive` so that all submodules are on the correct branch, etc.

Then from `base_agent/cpp`:

- `mkdir build`
- `cd build`
- `cmake ..`
- `cmake --build .`

## Run examples

- Full example using Tether Agent instance: `build/examples/tether_agent_example`
- Example of MsgPack + MQTT publish only: `build/examples/msgpack_publish_example`
- Example of MQTT (string message) publish only: `build/examples/publish_example`

## Install in your own CMake-based project

An example CMakeLists.txt that includes this library and dependent libraries:

```
cmake_minimum_required(VERSION 3.19)

project(My_TetherAgent)

add_executable(My_TetherAgent src/My_TetherAgent.cpp)
set_property(TARGET My_TetherAgent PROPERTY CXX_STANDARD 11)

add_subdirectory(./libs/tether/base_agent/cpp)
add_subdirectory(./libs/psn-cpp)

target_include_directories(My_TetherAgent PUBLIC ./libs/tether/base_agent/cpp/src)

target_link_libraries(My_TetherAgent PUBLIC TetherAgent msgpackc-cxx psnlib)
```
