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

## Build process so far:
- MQTT
  - `mkdir build`
  - `cd build`
  - `cmake .. -DPAHO_WITH_SSL=OFF` (for some reason this does not always "stick" if in cmake file!)
  - `cmake --build .`
  - Install, too?
- Msgpack
  - Must checkout `cpp_master` branch
  - Msgpack (currently?) needs both `cmake .` and `sudo cmake --install .` in `libs/msgpack` before the main cmake command will succeed