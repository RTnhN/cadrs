//! OCCT's placement conventions the kernel's DRAW-compatible builders share.

use super::math::*;

/// OCCT's x direction for a placement with only its main direction (`gp_Ax2(P, V)`).
pub fn ax2_x(v: &V3) -> V3 {
    let (a, b, c) = (v.x, v.y, v.z);
    let (aa, ba, ca) = (a.abs(), b.abs(), c.abs());
    let d = if ba <= aa && ba <= ca {
        if aa > ca {
            V3::new(-c, 0.0, a)
        } else {
            V3::new(c, 0.0, -a)
        }
    } else if aa <= ba && aa <= ca {
        if ba > ca {
            V3::new(0.0, -c, b)
        } else {
            V3::new(0.0, c, -b)
        }
    } else if aa > ba {
        V3::new(-b, a, 0.0)
    } else {
        V3::new(b, -a, 0.0)
    };
    d.normalize()
}
