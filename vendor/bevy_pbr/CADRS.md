# bevy_pbr 0.20.1, patched for cadrs

The crates.io release with one change, in the shaders `src/render/pbr_fragment.wesl`,
`pbr_prepass.wesl` and `pbr_prepass_functions.wesl`: each texture and sampler is named directly
in its `textureSample*` call, one statement per `BINDLESS` case, instead of being bound to a
`let` first.

WGSL doesn't allow a texture or sampler in a `let`. Tint (Chrome) and naga accept it, but
Safari's WebGPU compiler (macOS and iOS) rejects it ("'texture_2d<f32>' cannot be used as the
type of a 'let'"), so every `StandardMaterial` pipeline fails and the browser build quits on
its first frame in Safari. The bug came in with Bevy's move to WESL (bevyengine/bevy#25088,
first in 0.20.0); 0.19's WGSL shaders didn't have it. The change is bcanfield/bevy@e909449
from https://github.com/bevyengine/bevy/issues/26378, which applies to 0.20.1 as is.

The root `Cargo.toml` patches crates.io's bevy_pbr with this copy. Drop it once a Bevy release
has the fix (or update the copy when Bevy is upgraded).
