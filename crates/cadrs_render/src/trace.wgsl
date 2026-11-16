// The path tracer of `lib.rs` (`Renderer::sample`, `trace`, `ground_light`) as a compute
// shader: one invocation adds one sample to one pixel's sums. It follows the CPU code step by
// step, random numbers included (PCG32 in 64-bit arithmetic on pairs of u32), so the two
// renders agree to rounding. Keep them in step: a change to one is a change to both.

const PI: f32 = 3.14159265358979323846;

struct Light {
    // xyz the direction, w the cosine of the angular radius.
    dir: vec4<f32>,
    // xyz the radiance, w the solid angle.
    rad: vec4<f32>,
}

struct Params {
    // w: 0 perspective, 1 orthographic.
    eye: vec4<f32>,
    // w: tan(fov_y / 2) in perspective, the view's height in orthographic.
    fwd: vec4<f32>,
    // w: the aspect (width / height).
    right: vec4<f32>,
    // w: the ground plane's height.
    up: vec4<f32>,
    // w: the ray offset (eps).
    zenith: vec4<f32>,
    horizon: vec4<f32>,
    ground: vec4<f32>,
    // width, height, sample index, max bounces.
    dims: vec4<u32>,
    // seed (low, high), lights, flags (1 ground shadow, 2 the sky is the backdrop).
    misc: vec4<u32>,
    // first row of this dispatch, its rows, triangles.
    band: vec4<u32>,
    lights: array<Light, 8>,
}

@group(0) @binding(0) var<uniform> P: Params;
// Two per node: (min, first as bits), (max, count as bits).
@group(0) @binding(1) var<storage, read> nodes: array<vec4<f32>>;
// Three per triangle in BVH order: v0, e1, e2.
@group(0) @binding(2) var<storage, read> tris: array<vec4<f32>>;
// Four per triangle in BVH order: the vertex normals, then the face normal with the material
// index as bits in w.
@group(0) @binding(3) var<storage, read> shade: array<vec4<f32>>;
// Two per material: (base colour, metallic), (roughness, opacity, -, -).
@group(0) @binding(4) var<storage, read> materials: array<vec4<f32>>;
// Ten per pixel: model rgb, n_model, sky rgb, n_ground, occluded, open.
@group(0) @binding(5) var<storage, read_write> accum: array<f32>;
// Twelve per pixel: normal, albedo, position, footprint, hit, -.
@group(0) @binding(6) var<storage, read_write> guide: array<f32>;

// ---------------------------------------------------------------------------------------------
// 64-bit arithmetic on (low, high) and the random numbers (`math::Rng`).

fn mul32(a: u32, b: u32) -> vec2<u32> {
    let a0 = a & 0xffffu;
    let a1 = a >> 16u;
    let b0 = b & 0xffffu;
    let b1 = b >> 16u;
    let p00 = a0 * b0;
    let p01 = a0 * b1;
    let p10 = a1 * b0;
    let p11 = a1 * b1;
    let mid = p01 + p10;
    let mid_carry = select(0u, 0x10000u, mid < p01);
    let lo = p00 + (mid << 16u);
    let lo_carry = select(0u, 1u, lo < p00);
    return vec2<u32>(lo, p11 + (mid >> 16u) + mid_carry + lo_carry);
}

fn mul64(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    var r = mul32(a.x, b.x);
    r.y = r.y + a.x * b.y + a.y * b.x;
    return r;
}

fn add64(a: vec2<u32>, b: vec2<u32>) -> vec2<u32> {
    let lo = a.x + b.x;
    return vec2<u32>(lo, a.y + b.y + select(0u, 1u, lo < a.x));
}

fn shr64(a: vec2<u32>, n: u32) -> vec2<u32> {
    if n >= 32u {
        return vec2<u32>(a.y >> (n - 32u), 0u);
    }
    if n == 0u {
        return a;
    }
    return vec2<u32>((a.x >> n) | (a.y << (32u - n)), a.y >> n);
}

fn mix64(z0: vec2<u32>) -> vec2<u32> {
    var z = add64(z0, vec2<u32>(0x7F4A7C15u, 0x9E3779B9u));
    z = mul64(z ^ shr64(z, 30u), vec2<u32>(0x1CE4E5B9u, 0xBF58476Du));
    z = mul64(z ^ shr64(z, 27u), vec2<u32>(0x133111EBu, 0x94D049BBu));
    return z ^ shr64(z, 31u);
}

struct Rng {
    state: vec2<u32>,
    inc: vec2<u32>,
}

fn next_u32(r: ptr<function, Rng>) -> u32 {
    let old = (*r).state;
    (*r).state = add64(mul64(old, vec2<u32>(0x4C957F2Du, 0x5851F42Du)), (*r).inc);
    let xorshifted = shr64(shr64(old, 18u) ^ old, 27u).x;
    let rot = shr64(old, 59u).x;
    return (xorshifted >> rot) | (xorshifted << ((32u - rot) & 31u));
}

fn rng_new(seed: vec2<u32>, pixel: u32, smp: u32) -> Rng {
    let p = mul64(vec2<u32>(pixel, 0u), vec2<u32>(0x4F6CDD1Du, 0x2545F491u));
    let a = mix64(seed ^ mix64(p ^ mix64(vec2<u32>(smp, 0u))));
    let m = mix64(a ^ vec2<u32>(0x94B95BDBu, 0xDA3E39CBu));
    // (m << 1) | 1
    let inc = vec2<u32>((m.x << 1u) | 1u, (m.y << 1u) | (m.x >> 31u));
    var r = Rng(vec2<u32>(0u, 0u), inc);
    next_u32(&r);
    r.state = add64(r.state, a);
    next_u32(&r);
    return r;
}

fn rf(r: ptr<function, Rng>) -> f32 {
    return f32(next_u32(r) >> 8u) * (1.0 / 16777216.0);
}

// ---------------------------------------------------------------------------------------------
// Vectors (`math::V3`)

fn nrm(v: vec3<f32>) -> vec3<f32> {
    let l = sqrt(dot(v, v));
    if l > 0.0 {
        return v / l;
    }
    return vec3<f32>(0.0, 0.0, 1.0);
}

fn lerp3(a: vec3<f32>, b: vec3<f32>, t: f32) -> vec3<f32> {
    return a + (b - a) * t;
}

fn lum(c: vec3<f32>) -> f32 {
    return 0.2126 * c.x + 0.7152 * c.y + 0.0722 * c.z;
}

fn max_elem(v: vec3<f32>) -> f32 {
    return max(max(v.x, v.y), v.z);
}

struct Basis {
    t: vec3<f32>,
    b: vec3<f32>,
}

fn basis(n: vec3<f32>) -> Basis {
    let s = select(-1.0, 1.0, n.z >= 0.0);
    let a = -1.0 / (s + n.z);
    let b = n.x * n.y * a;
    return Basis(
        vec3<f32>(1.0 + s * n.x * n.x * a, s * b, -s * n.x),
        vec3<f32>(b, s + n.y * n.y * a, -n.y),
    );
}

// ---------------------------------------------------------------------------------------------
// The BVH (`bvh::Bvh`)

struct Hit {
    ok: bool,
    tri: u32,
    t: f32,
    u: f32,
    v: f32,
}

fn no_hit() -> Hit {
    return Hit(false, 0u, 0.0, 0.0, 0.0);
}

// The entry distance, or -1 for a miss.
fn box_hit(i: u32, o: vec3<f32>, inv: vec3<f32>, tmax: f32) -> f32 {
    let mn = nodes[2u * i].xyz;
    let mx = nodes[2u * i + 1u].xyz;
    let t1 = (mn - o) * inv;
    let t2 = (mx - o) * inv;
    let tmin = max(max_elem(min(t1, t2)), 0.0);
    let tf = max(t1, t2);
    let tfar = min(min(min(tf.x, tf.y), tf.z), tmax);
    if tmin <= tfar {
        return tmin;
    }
    return -1.0;
}

fn traverse(o: vec3<f32>, d: vec3<f32>, tmax0: f32, any_hit: bool) -> Hit {
    var best = no_hit();
    if P.band.z == 0u {
        return best;
    }
    var tmax = tmax0;
    let inv = vec3<f32>(1.0 / d.x, 1.0 / d.y, 1.0 / d.z);
    if box_hit(0u, o, inv, tmax) < 0.0 {
        return best;
    }
    var stack: array<u32, 64>;
    var sp = 1u;
    stack[0] = 0u;
    while sp > 0u {
        sp -= 1u;
        let ni = stack[sp];
        let first = bitcast<u32>(nodes[2u * ni].w);
        let count = bitcast<u32>(nodes[2u * ni + 1u].w);
        if count > 0u {
            for (var k = first; k < first + count; k++) {
                let v0 = tris[3u * k].xyz;
                let e1 = tris[3u * k + 1u].xyz;
                let e2 = tris[3u * k + 2u].xyz;
                let p = cross(d, e2);
                let det = dot(e1, p);
                if abs(det) < 1e-12 {
                    continue;
                }
                let idet = 1.0 / det;
                let s = o - v0;
                let u = dot(s, p) * idet;
                if u < 0.0 || u > 1.0 {
                    continue;
                }
                let q = cross(s, e1);
                let v = dot(d, q) * idet;
                if v < 0.0 || u + v > 1.0 {
                    continue;
                }
                let dist = dot(e2, q) * idet;
                if dist > 0.0 && dist < tmax {
                    tmax = dist;
                    best = Hit(true, k, dist, u, v);
                    if any_hit {
                        return best;
                    }
                }
            }
            continue;
        }
        let l = first;
        let r = first + 1u;
        let hl = box_hit(l, o, inv, tmax);
        let hr = box_hit(r, o, inv, tmax);
        if hl >= 0.0 && hr >= 0.0 {
            var near = l;
            var far = r;
            if hl > hr {
                near = r;
                far = l;
            }
            if sp + 2u <= 64u {
                stack[sp] = far;
                stack[sp + 1u] = near;
                sp += 2u;
            }
        } else if hl >= 0.0 {
            stack[sp] = l;
            sp += 1u;
        } else if hr >= 0.0 {
            stack[sp] = r;
            sp += 1u;
        }
    }
    return best;
}

fn intersect(o: vec3<f32>, d: vec3<f32>) -> Hit {
    return traverse(o, d, 3.40282347e38, false);
}

fn occluded(o: vec3<f32>, d: vec3<f32>) -> bool {
    return traverse(o, d, 3.40282347e38, true).ok;
}

// ---------------------------------------------------------------------------------------------
// The surface model (`bsdf::Bsdf`)

struct Bsdf {
    diffuse: vec3<f32>,
    f0: vec3<f32>,
    alpha: f32,
    p_spec: f32,
}

fn schlick(f0: vec3<f32>, c: f32) -> vec3<f32> {
    let m = clamp(1.0 - c, 0.0, 1.0);
    let m5 = m * m * m * m * m;
    return f0 + (vec3<f32>(1.0) - f0) * m5;
}

fn bsdf_new(base: vec3<f32>, metallic0: f32, roughness: f32, n_dot_o: f32) -> Bsdf {
    let metallic = clamp(metallic0, 0.0, 1.0);
    let f0 = lerp3(vec3<f32>(0.04), base, metallic);
    let diffuse = base * (1.0 - metallic);
    let r = clamp(roughness, 0.045, 1.0);
    let alpha = r * r;
    let ws = lum(schlick(f0, n_dot_o));
    let wd = lum(diffuse);
    var p_spec = 1.0;
    if wd > 1e-6 {
        p_spec = clamp(ws / (ws + wd), 0.15, 0.9);
    }
    return Bsdf(diffuse, f0, alpha, p_spec);
}

fn ggx_d(b: Bsdf, nh: f32) -> f32 {
    let a2 = b.alpha * b.alpha;
    let t = nh * nh * (a2 - 1.0) + 1.0;
    return a2 / (PI * t * t);
}

fn ggx_g1(b: Bsdf, nv: f32) -> f32 {
    let a2 = b.alpha * b.alpha;
    return 2.0 * nv / (nv + sqrt(a2 + (1.0 - a2) * nv * nv));
}

fn bsdf_eval(b: Bsdf, n: vec3<f32>, wo: vec3<f32>, wi: vec3<f32>) -> vec3<f32> {
    let no = dot(n, wo);
    let ni = dot(n, wi);
    if no <= 0.0 || ni <= 0.0 {
        return vec3<f32>(0.0);
    }
    let h = nrm(wo + wi);
    let nh = max(dot(n, h), 0.0);
    let f = schlick(b.f0, max(dot(wo, h), 0.0));
    let spec = f * (ggx_d(b, nh) * ggx_g1(b, no) * ggx_g1(b, ni) / (4.0 * no * ni));
    let diff = (vec3<f32>(1.0) - f) * b.diffuse * (1.0 / PI);
    return diff + spec;
}

fn bsdf_pdf(b: Bsdf, n: vec3<f32>, wo: vec3<f32>, wi: vec3<f32>) -> f32 {
    let ni = dot(n, wi);
    if ni <= 0.0 || dot(n, wo) <= 0.0 {
        return 0.0;
    }
    let h = nrm(wo + wi);
    let nh = max(dot(n, h), 0.0);
    let spec = ggx_d(b, nh) * nh / (4.0 * max(abs(dot(wo, h)), 1e-6));
    let diff = ni / PI;
    return b.p_spec * spec + (1.0 - b.p_spec) * diff;
}

struct Sampled {
    ok: bool,
    wi: vec3<f32>,
    f: vec3<f32>,
    pdf: f32,
}

fn bsdf_sample(b: Bsdf, n: vec3<f32>, wo: vec3<f32>, rng: ptr<function, Rng>) -> Sampled {
    let fr = basis(n);
    let u1 = rf(rng);
    let u2 = rf(rng);
    var wi: vec3<f32>;
    if rf(rng) < b.p_spec {
        let a2 = b.alpha * b.alpha;
        let cos_t = sqrt(max((1.0 - u1) / (1.0 + (a2 - 1.0) * u1), 0.0));
        let sin_t = sqrt(max(1.0 - cos_t * cos_t, 0.0));
        let phi = 2.0 * PI * u2;
        let h = fr.t * (sin_t * cos(phi)) + fr.b * (sin_t * sin(phi)) + n * cos_t;
        wi = nrm(h * (2.0 * dot(wo, h)) - wo);
    } else {
        let r = sqrt(u1);
        let phi = 2.0 * PI * u2;
        wi = nrm(fr.t * (r * cos(phi)) + fr.b * (r * sin(phi)) + n * sqrt(max(1.0 - u1, 0.0)));
    }
    let pdf = bsdf_pdf(b, n, wo, wi);
    if pdf <= 1e-8 {
        return Sampled(false, wi, vec3<f32>(0.0), 0.0);
    }
    return Sampled(true, wi, bsdf_eval(b, n, wo, wi), pdf);
}

// ---------------------------------------------------------------------------------------------
// The environment (`env::Environment`)

fn dome(d: vec3<f32>) -> vec3<f32> {
    if d.z >= 0.0 {
        return lerp3(P.horizon.xyz, P.zenith.xyz, sqrt(d.z));
    }
    return lerp3(P.horizon.xyz, P.ground.xyz, min(-d.z * 8.0, 1.0));
}

fn light_sample(i: u32, u1: f32, u2: f32) -> vec3<f32> {
    let dir = P.lights[i].dir.xyz;
    let cos_max = P.lights[i].dir.w;
    let cos_t = 1.0 - u1 * (1.0 - cos_max);
    let sin_t = sqrt(max(1.0 - cos_t * cos_t, 0.0));
    let phi = 2.0 * PI * u2;
    let fr = basis(dir);
    return nrm(fr.t * (sin_t * cos(phi)) + fr.b * (sin_t * sin(phi)) + dir * cos_t);
}

fn pick_light(rng: ptr<function, Rng>) -> u32 {
    let n = P.misc.z;
    return min(u32(rf(rng) * f32(n)), n - 1u);
}

// ---------------------------------------------------------------------------------------------
// The path (`Renderer::trace`, `ground_light`)

var<private> g_guide: array<f32, 12>;
var<private> g_want_guide: bool;

fn footprint(t: f32) -> f32 {
    if P.eye.w == 0.0 {
        return t * 2.0 * P.fwd.w / f32(P.dims.y);
    }
    return P.fwd.w / f32(P.dims.y);
}

fn trace(o0: vec3<f32>, d0: vec3<f32>, first: Hit, rng: ptr<function, Rng>) -> vec3<f32> {
    let n_lights = P.misc.z;
    let nl = f32(n_lights);
    let eps = P.zenith.w;
    var o = o0;
    var d = d0;
    var radiance = vec3<f32>(0.0);
    var throughput = vec3<f32>(1.0);
    var hit = first;
    var last_pdf = 0.0;
    var specular_bounce = true;
    var bounce = 0u;
    var passes = 0u;
    while bounce < P.dims.w {
        if !hit.ok {
            var c = dome(d);
            for (var i = 0u; i < n_lights; i++) {
                if dot(d, P.lights[i].dir.xyz) >= P.lights[i].dir.w {
                    var w = 1.0;
                    if !specular_bounce {
                        let pl = 1.0 / (nl * P.lights[i].rad.w);
                        w = last_pdf * last_pdf / (last_pdf * last_pdf + pl * pl);
                    }
                    c += P.lights[i].rad.xyz * w;
                }
            }
            radiance += throughput * c;
            break;
        }
        let k = hit.tri;
        let p = o + d * hit.t;
        var ng = shade[4u * k + 3u].xyz;
        let mi = bitcast<u32>(shade[4u * k + 3u].w);
        let na = shade[4u * k].xyz;
        let nb = shade[4u * k + 1u].xyz;
        let nc = shade[4u * k + 2u].xyz;
        var ns = ng;
        let zero = vec3<f32>(0.0);
        if !(all(na == zero) || all(nb == zero) || all(nc == zero)) {
            ns = nrm(na * (1.0 - hit.u - hit.v) + nb * hit.u + nc * hit.v);
        }
        if dot(ng, d) > 0.0 {
            ng = -ng;
        }
        if dot(ns, ng) < 0.0 {
            ns = -ns;
        }
        let wo = -d;
        if dot(ns, wo) <= 0.0 {
            ns = nrm(ns + ng * (0.01 - dot(ns, wo)) * 2.0);
            if dot(ns, wo) <= 0.0 {
                ns = ng;
            }
        }
        let m0 = materials[2u * mi];
        let m1 = materials[2u * mi + 1u];
        let base = m0.xyz;
        let opacity = m1.y;
        if opacity < 1.0 && rf(rng) >= opacity && passes < 8u {
            passes += 1u;
            o = p + d * eps;
            hit = intersect(o, d);
            continue;
        }
        if g_want_guide {
            g_want_guide = false;
            g_guide[0] = ns.x;
            g_guide[1] = ns.y;
            g_guide[2] = ns.z;
            g_guide[3] = base.x;
            g_guide[4] = base.y;
            g_guide[5] = base.z;
            g_guide[6] = p.x;
            g_guide[7] = p.y;
            g_guide[8] = p.z;
            g_guide[9] = footprint(hit.t);
            g_guide[10] = 1.0;
        }
        let b = bsdf_new(base, m0.w, m1.x, dot(ns, wo));
        let origin = p + ng * eps;
        if n_lights > 0u {
            let li = pick_light(rng);
            let u1 = rf(rng);
            let u2 = rf(rng);
            let wi = light_sample(li, u1, u2);
            if dot(wi, ng) > 0.0 && dot(wi, ns) > 0.0 {
                let f = bsdf_eval(b, ns, wo, wi);
                if max_elem(f) > 0.0 && !occluded(origin, wi) {
                    let pl = 1.0 / (nl * P.lights[li].rad.w);
                    let pb = bsdf_pdf(b, ns, wo, wi);
                    let w = pl * pl / (pl * pl + pb * pb);
                    radiance += throughput * (f * P.lights[li].rad.xyz) * (dot(wi, ns) * w / pl);
                }
            }
        }
        let s = bsdf_sample(b, ns, wo, rng);
        if !s.ok {
            break;
        }
        if dot(s.wi, ng) <= 0.0 {
            break;
        }
        throughput = throughput * s.f * (dot(s.wi, ns) / s.pdf);
        last_pdf = s.pdf;
        specular_bounce = false;
        bounce += 1u;
        if bounce >= 3u {
            let q = clamp(max_elem(throughput), 0.05, 0.95);
            if rf(rng) >= q {
                break;
            }
            throughput *= 1.0 / q;
        }
        o = origin;
        d = s.wi;
        hit = intersect(o, d);
    }
    let m = max_elem(radiance);
    if m > 24.0 {
        return radiance * (24.0 / m);
    }
    return radiance;
}

// (occluded, open) light on the ground at `p`.
fn ground_light(p: vec3<f32>, rng: ptr<function, Rng>) -> vec2<f32> {
    let n = vec3<f32>(0.0, 0.0, 1.0);
    let o = p + n * P.zenith.w;
    var occ = 0.0;
    var open = 0.0;
    let fr = basis(n);
    for (var i = 0; i < 4; i++) {
        let u1 = rf(rng);
        let u2 = rf(rng);
        let r = sqrt(u1);
        let phi = 2.0 * PI * u2;
        let d = fr.t * (r * cos(phi)) + fr.b * (r * sin(phi)) + n * sqrt(max(1.0 - u1, 0.0));
        let l = lum(dome(d)) * PI / 4.0;
        open += l;
        if !occluded(o, d) {
            occ += l;
        }
    }
    let n_lights = P.misc.z;
    if n_lights > 0u {
        let k = pick_light(rng);
        let u1 = rf(rng);
        let u2 = rf(rng);
        let wi = light_sample(k, u1, u2);
        if wi.z > 0.0 {
            let c = lum(P.lights[k].rad.xyz) * wi.z * P.lights[k].rad.w * f32(n_lights);
            open += c;
            if !occluded(o, wi) {
                occ += c;
            }
        }
    }
    return vec2<f32>(occ, open);
}

@compute @workgroup_size(8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let w = P.dims.x;
    let h = P.dims.y;
    let s = P.dims.z;
    let x = gid.x;
    let y = P.band.x + gid.y;
    if x >= w || gid.y >= P.band.y || y >= h {
        return;
    }
    let pixel = y * w + x;
    var rng = rng_new(P.misc.xy, pixel, s);
    var jx = 0.5;
    var jy = 0.5;
    if s != 0u {
        jx = rf(&rng);
        jy = rf(&rng);
    }
    let aspect = P.right.w;
    let sx = (f32(x) + jx) / f32(w);
    let sy = (f32(y) + jy) / f32(h);
    let px = 2.0 * sx - 1.0;
    let py = 1.0 - 2.0 * sy;
    var o: vec3<f32>;
    var d: vec3<f32>;
    if P.eye.w == 0.0 {
        let t = P.fwd.w;
        o = P.eye.xyz;
        d = nrm(P.fwd.xyz + P.right.xyz * (px * t * aspect) + P.up.xyz * (py * t));
    } else {
        let height = P.fwd.w;
        o = P.eye.xyz + P.right.xyz * (px * height * aspect / 2.0) + P.up.xyz * (py * height / 2.0);
        d = P.fwd.xyz;
    }
    let hit = intersect(o, d);
    var ground_t = -1.0;
    if (P.misc.w & 1u) != 0u && d.z < -1e-6 {
        let t = (P.up.w - o.z) / d.z;
        if t > 0.0 {
            ground_t = t;
        }
    }
    var sky = vec3<f32>(0.0);
    if (P.misc.w & 2u) != 0u {
        sky = dome(d);
    }
    let a = pixel * 10u;
    if hit.ok && (ground_t < 0.0 || hit.t < ground_t) {
        g_want_guide = s == 0u;
        let radiance = trace(o, d, hit, &rng);
        accum[a] += radiance.x;
        accum[a + 1u] += radiance.y;
        accum[a + 2u] += radiance.z;
        accum[a + 3u] += 1.0;
        if s == 0u && !g_want_guide {
            let g = pixel * 12u;
            for (var i = 0u; i < 11u; i++) {
                guide[g + i] = g_guide[i];
            }
        }
    } else if ground_t >= 0.0 {
        let l = ground_light(o + d * ground_t, &rng);
        accum[a + 4u] += sky.x;
        accum[a + 5u] += sky.y;
        accum[a + 6u] += sky.z;
        accum[a + 7u] += 1.0;
        accum[a + 8u] += l.x;
        accum[a + 9u] += l.y;
    } else {
        accum[a + 4u] += sky.x;
        accum[a + 5u] += sky.y;
        accum[a + 6u] += sky.z;
    }
}
