import re

with open("kernel/src/panic.rs", "r") as f:
    content = f.read()

content = content.replace("#[cfg(all(not(test), not(feature = \"std\")))]", "#[cfg(all(not(test), any(target_os = \"none\", target_os = \"uefi\")))]")

with open("kernel/src/panic.rs", "w") as f:
    f.write(content)
