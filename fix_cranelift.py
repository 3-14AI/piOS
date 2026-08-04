import re

with open("libs/compiler_frontend/src/cranelift_backend.rs", "r") as f:
    content = f.read()

# Remove any old EntityRef imports
content = re.sub(r'use cranelift_codegen::cranelift_entity::EntityRef;\n?', '', content)
content = re.sub(r'use cranelift_codegen::entity::EntityRef;\n?', '', content)

# Add correct EntityRef import
import_line = "use cranelift_codegen::entity::EntityRef;\n"
if "cranelift_frontend" in content:
    content = content.replace("use cranelift_frontend", import_line + "use cranelift_frontend")

# Replace Variable::new, Variable::with_u32, Variable::from_u32
def repl(m):
    var = m.group(1).replace(" as u32", "").replace(" as usize", "")
    return f"Variable::from_u32({var} as u32)"

content = re.sub(r'Variable::(?:new|with_u32|from_u32)\((.*?)\)', repl, content)

with open("libs/compiler_frontend/src/cranelift_backend.rs", "w") as f:
    f.write(content)
