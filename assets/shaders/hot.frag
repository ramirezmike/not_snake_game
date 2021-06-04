#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec3 v_WorldNormal;
layout(location = 0) out vec4 o_Target;

layout(set = 2, binding = 0) uniform TimeUniform_value {
    float time;
};

vec3 make_beam(
    in vec2 uv, 
    in float time, 
    in float wiggle1,
    in float wiggle2,
    in float wiggle3,
    in float verticalSize
) {
    float verticalAlignment = 1.0;
    vec2 p = (uv * 2.0 - verticalAlignment) * verticalSize;

    vec2 sfunc = 
        vec2(0.0, 
             p.y
             // movement to the right
             + wiggle1 * sin(uv.x * 10.0 - time * wiggle2 + cos(time * 2.0))

             // movement to the left
             + wiggle3 * cos(uv.x * 25.0 + time * 8.0));

    // makes glow on each end
//  sfunc.y *= uv.x*2.0+0.05;       // left side
//  sfunc.y *= 2.0 - uv.x*2.0+0.05; // right side

    sfunc.y /= 0.1 ; // affects brightness of everything

    vec3 color = vec3(abs(sfunc.y));

    // affects how much color spreads from beam
    color = pow(color, vec3(-0.8)); 

    // actually sets color
    color *= vec3(0.8, 0.8, 0.0);
    return color;
}

void main() {
    vec3 beam_1 = make_beam(v_Uv, time, 6.0, 2.0, 2.0, 12.0);
    vec3 beam_2 = make_beam(v_Uv, time, 5.0, 3.0, 2.0, 13.0);

    o_Target = vec4(beam_1, 1.0);
    o_Target = mix(o_Target, vec4(beam_2, 1.0), 0.5);

    if (o_Target.x < 0.2) {
        discard;
    }
}
