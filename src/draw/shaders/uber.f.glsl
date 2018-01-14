#version 410

const float PI = 3.14159265359;
const float F0_REFLECTIVITY = 0.0337;

uniform sampler2D normal_tex;
uniform sampler2D albedo_tex;
uniform sampler2D knobs_tex;

uniform samplerCube irradiance_map;
uniform samplerCube radiance_map;
uniform sampler2D integrated_brdf_map;

uniform sampler2DShadow shadow_depth;

layout(std140) uniform transform {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec4 eye_pos;
    float clip_offset;
};

layout(std140) uniform params {
    mat4 sun_matrix;
    vec4 sun_color;
    float sun_in_env;

    float gamma;
    float exposure;
};

in vec3 I_POS;
in vec3 I_NORM;
in vec2 I_TEX;
in vec3 I_TAN;
in vec3 I_BITAN;
out vec4 f_color;

vec3 fresnel_schlick(float cos_theta, vec3 f_0) {
    return f_0 + (1 - f_0) * pow(1 - cos_theta, 5);
}

float distribution_ggx(float cos_theta, float alpha) {
    float alpha2 = alpha * alpha;
    float cos_theta2 = cos_theta * cos_theta;

    float denom = cos_theta2 * (alpha2 - 1) + 1;
    denom = PI * denom * denom;

    return alpha2 / denom;
}

float geometry_schlick_ggx(float cos_theta, float alpha) {
    float k = alpha / 2;
    float denom = cos_theta * (1 - k) + k;
    return cos_theta / denom;
}

float geometry_smith(float NdotV, float NdotL, float alpha) {
    float ggx2 = geometry_schlick_ggx(NdotV, alpha);
    float ggx1 = geometry_schlick_ggx(NdotL, alpha);
    return ggx1 * ggx2;
}

vec3 light_contrib(
    float NdotL,
    float NdotV,
    float NdotH,
    float VdotH,
    vec3 radiance,
    vec3 albedo,
    float alpha,
    float metalness) {
    
    // fresnel reflectance
    vec3 F0 = mix(vec3(F0_REFLECTIVITY), albedo, metalness);

    // brdf
    float ndf = distribution_ggx(NdotH, alpha);
    float geo = geometry_smith(NdotV, NdotL, alpha);
    vec3 fres = fresnel_schlick(NdotV, F0);
    vec3 specular_brdf = ndf * geo * fres / (4 * NdotV * NdotL);
    vec3 diffuse_brdf = NdotL * albedo * (vec3(1) - fres) * (1 - metalness);

    // outgoing radiance
    return (diffuse_brdf + specular_brdf) * radiance;
}

void main() {
    // normal mapping
    vec3 normal_map = texture(normal_tex, I_TEX).rgb * 2 - 1;
    vec3 norm = mat3(I_TAN, I_BITAN, I_NORM) * normal_map;

    // material params
    vec3 albedo = texture(albedo_tex, I_TEX).rgb;
    vec3 knobs = texture(knobs_tex, I_TEX).rgb;
    float metalness = knobs.r;
    float roughness = knobs.g;
    float alpha = roughness * roughness;
    float solidness = knobs.b;

    // imortant vectors
    vec3 N = normalize(norm);
    vec3 V = normalize(eye_pos.xyz - I_POS);
    float NdotV = abs(dot(N, V));
    vec3 R = V - 2 * NdotV * N;
    NdotV = clamp(NdotV, 0.01, 1.0);

    // outgoing radiance
    vec3 lum = vec3(0);

    // IBL
    // indirect diffuse
    lum += texture(irradiance_map, N).rgb * albedo * (1 - metalness);
    vec2 env_brdf = texture(integrated_brdf_map, vec2(NdotV, roughness)).rg;
    lum += texture(radiance_map, R).rgb * (albedo * env_brdf.r + vec3(env_brdf.g));

    // sun shadow
    vec4 sun_frag_pos = sun_matrix * vec4(I_POS, 1);
    vec3 sun_frag_uv = sun_frag_pos.xyz / sun_frag_pos.w * 0.5 + 0.5; // position in shadow buffer
    float shadow_level = 1 /*texture(shadow_depth, sun_frag_uv)*/ - sun_in_env;

    // sun vectors
    vec3 sun_L = -(sun_matrix * vec4(0, 0, -1, 0)).xyz;
    vec3 sun_H = normalize(V + sun_L); // halfway vector
    float sun_NdotL = clamp(dot(N, sun_L), 0.01, 1.0);
    float sun_NdotH = clamp(dot(N, sun_H), 0.0, 1.0);
    float sun_VdotH = clamp(dot(V, sun_H), 0.0, 1.0);

    lum += shadow_level * light_contrib(
        sun_NdotL,
        NdotV,
        sun_NdotH,
        sun_VdotH,
        sun_color.rgb * sun_color.a,
        albedo,
        alpha,
        metalness);

    // hdr to ldr  
    vec3 mapped = vec3(1.0) - exp(-lum * exposure);
    //mapped = mix(mapped, albedo, solidness); // make solid
    mapped = pow(mapped, vec3(1 / gamma));

    f_color = vec4(mapped, 1.0);
}
