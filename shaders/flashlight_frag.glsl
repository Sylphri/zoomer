#version 330 core

in vec2 texCoord;
in vec4 pos;

uniform sampler2D texture1;
uniform vec2 cursorPos;
uniform float radius;
uniform vec2 resolution;

void main()
{
    // gl_FragColor = vec4(cursorPos.x, cursorPos.y, 0.0, 1.0);
    if (pow((pos.x - cursorPos.x) * (resolution.x / resolution.y), 2.0) + pow(-pos.y - cursorPos.y, 2.0) > radius * radius)
        gl_FragColor = mix(texture(texture1, vec2(texCoord.x, -texCoord.y)), vec4(0.0, 0.0, 0.0, 1.0), 0.7);
    else
        gl_FragColor = texture(texture1, vec2(texCoord.x, -texCoord.y));
    // gl_FragColor = vec4(pos.x, pos.y, 0.0, 1.0);
}
