BindGroups passed to shader:
- Group 0: model related uniforms, e.g., model matrix.
- Group 1: camera related uniforms, e.g., view matrix, projection matrix and camera position.
- Group 2: concrete material related uniforms, e.g., albedo texture / color for pbr shader.
- Group 3: light sources related uniforms.

Group 0, 1, 3 will be created by the renderer automatically.

Group 2 will be created by the material struct.