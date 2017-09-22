#version 410

const float PI = 3.14159265359;

uniform sampler2D normal_tex;
uniform sampler2D albedo_tex;
uniform sampler2D metalness_tex;
uniform sampler2D roughness_tex;

layout(std140) uniform params {
    vec4 ambient;
    // float gamma;
    // float exposure;
};

layout(std140) uniform transform {
    mat4 model;
    mat4 view;
    mat4 proj;
    vec4 eye_pos;
    float xoffset;
};

struct Light {
    vec4 pos;
    vec4 color;
};

layout(std140) uniform lights_layout {
    Light lights[LIGHT_COUNT];
};

in vec3 I_POS;
in vec3 I_NORM;
in vec2 I_TEX;
in vec3 I_TAN;
in vec3 I_BITAN;
out vec4 f_lum;

vec3 fresnelSchlick(float cosTheta, vec3 F0) {
    return F0 + (1.0 - F0) * pow(1.0 - cosTheta, 5.0);
}

float distributionGGX(vec3 N, vec3 H, float roughness) {
    float a = roughness * roughness;
    a = a * a;
    float n_dot_h = max(dot(N, H), 0.0);
    float n_dot_h2 = n_dot_h * n_dot_h;
    
    float denom = (n_dot_h2 * (a - 1.0) + 1.0);
    denom = PI * denom * denom;
    
    return a / denom;
}

float geometrySchlickGGX(float n_dot_v, float roughness) {
    float rough_more = (roughness + 1.0);
    float k = (rough_more * rough_more) / 8.0;

    float denom = n_dot_v * (1.0 - k) + k;
    
    return n_dot_v / denom;
}

float geometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
    float n_dot_v = max(dot(N, V), 0.0);
    float n_dot_l = max(dot(N, L), 0.0);
    float ggx2 = geometrySchlickGGX(n_dot_v, roughness);
    float ggx1 = geometrySchlickGGX(n_dot_l, roughness);
    
    return ggx1 * ggx2;
}

void main() {
    vec3 normal_map = texture(normal_tex, I_TEX).rgb * 2 - 1;
    vec3 norm = mat3(I_TAN, I_BITAN, I_NORM) * normal_map;

    vec3 albedo = texture(albedo_tex, I_TEX).rgb;
    float roughness = texture(roughness_tex, I_TEX).r;
    float metalness = texture(metalness_tex, I_TEX).r;

    vec3 F0 = vec3(0.04);
    F0 = mix(F0, albedo, metalness);

    vec3 N = normalize(norm);
    vec3 V = normalize(eye_pos.xyz - I_POS);

    // AMBIENT
    vec3 lum = ambient.xyz * ambient.a * albedo; // * ao;

    for (int i = 0; i < LIGHT_COUNT; i++) {
        Light light = lights[i];
        vec3 lpos = light.pos.xyz;

        vec3 L = normalize(lpos - I_POS);
        vec3 H = normalize(V + L);
        float dist = length(lpos - I_POS);
        vec3 radiance = light.color.rgb * light.color.a / (dist * dist);
        
        // brdf
        float NDF = distributionGGX(N, H, roughness);        
        float G = geometrySmith(N, V, L, roughness);      
        vec3 F = fresnelSchlick(max(dot(H, V), 0.0), F0);       
        
        vec3 kS = F;
        vec3 kD = vec3(1.0) - kS;
        kD *= 1.0 - metalness;     
        
        vec3 nominator = NDF * G * F;
        float denominator = 4 * max(dot(N, V), 0.0) * max(dot(N, L), 0.0) + 0.001; 
        vec3 brdf = nominator / denominator;
            
        // add to outgoing radiance Lo
        float n_dot_l = max(dot(N, L), 0.0);                
        lum += (kD * albedo / PI + brdf) * radiance * n_dot_l;
    }

    // OUT
    f_lum = vec4(lum, 1);
} 