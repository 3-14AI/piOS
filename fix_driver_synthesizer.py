import re

with open('userland/services/driver_synthesizer/src/lib.rs', 'r') as f:
    content = f.read()

content = content.replace('#[cfg(any(test, not(target_arch = "wasm32")))]\nmod tests {', '#[cfg(test)]\nmod tests {')

with open('userland/services/driver_synthesizer/src/lib.rs', 'w') as f:
    f.write(content)
