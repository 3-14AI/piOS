with open("todo.md", "r") as f:
    content = f.read()

content = content.replace("[Phase 5] WP-049", "[x] [Phase 5] WP-049")

with open("todo.md", "w") as f:
    f.write(content)
