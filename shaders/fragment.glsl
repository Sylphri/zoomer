#version 330 core

in vec2 texCoord;

uniform sampler2D texture1;

void main()
{
    gl_FragColor = texture(texture1, vec2(texCoord.x, -texCoord.y));
}

