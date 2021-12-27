## TODO

- [x] Demo: Basic MQTT publish working
- [x] Demo: basic MessagePack serialisation
- [ ] Demo: Get basic MessagePack serialised message into MQTT body
- [ ] Build actual library that can be included from other CPP projects, rather than sitting inside `examples` folder - using CMake again
- [ ] Merge this project into the monorepo

## Third-party libraries used

- MQTT: https://github.com/eclipse/paho.mqtt.cpp
- MessagePack: https://github.com/msgpack/msgpack-c/tree/cpp_master
- PSN CPP: https://github.com/vyv/psn-cpp

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
