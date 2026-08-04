import re

with open("libs/compiler_frontend/src/cranelift_backend.rs", "r") as f:
    content = f.read()

content = content.replace("use cranelift_codegen::entity::EntityRef;", "use cranelift_codegen::entity::EntityRef;\nuse cranelift_codegen::settings::Configurable;\nuse alloc::sync::Arc;")
content = content.replace("alloc::boxed::Box<dyn TargetIsa>", "Arc<dyn TargetIsa>")

with open("libs/compiler_frontend/src/cranelift_backend.rs", "w") as f:
    f.write(content)
