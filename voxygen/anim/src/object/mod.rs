pub mod beam;
pub mod idle;
pub mod shoot;

// Reexports
pub use self::{beam::BeamAnimation, idle::IdleAnimation, shoot::ShootAnimation};

use super::{make_bone, vek::*, FigureBoneData, Skeleton};
use common::comp::{self};
use core::convert::TryFrom;

pub type Body = comp::object::Body;

skeleton_impls!(struct ObjectSkeleton {
    + bone0,
    + bone1,
});

impl Skeleton for ObjectSkeleton {
    type Attr = SkeletonAttr;
    type Body = Body;

    const BONE_COUNT: usize = 2;
    #[cfg(feature = "use-dyn-lib")]
    const COMPUTE_FN: &'static [u8] = b"object_compute_mats\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "object_compute_mats")]
    fn compute_matrices_inner(
        &self,
        base_mat: Mat4<f32>,
        buf: &mut [FigureBoneData; super::MAX_BONE_COUNT],
    ) -> Vec3<f32> {
        let bone0_mat = base_mat * Mat4::<f32>::from(self.bone0);

        *(<&mut [_; Self::BONE_COUNT]>::try_from(&mut buf[0..Self::BONE_COUNT]).unwrap()) = [
            make_bone(bone0_mat * Mat4::scaling_3d(1.0 / 11.0)),
            make_bone(Mat4::<f32>::from(self.bone1) * Mat4::scaling_3d(1.0 / 11.0)), /* Decorellated from ori */
        ];
        Vec3::unit_z() * 0.5
    }
}

pub struct SkeletonAttr {
    bone0: (f32, f32, f32),
    bone1: (f32, f32, f32),
}

impl<'a> std::convert::TryFrom<&'a comp::Body> for SkeletonAttr {
    type Error = ();

    fn try_from(body: &'a comp::Body) -> Result<Self, Self::Error> {
        match body {
            comp::Body::Object(body) => Ok(SkeletonAttr::from(body)),
            _ => Err(()),
        }
    }
}

impl Default for SkeletonAttr {
    fn default() -> Self {
        Self {
            bone0: (0.0, 0.0, 0.0),
            bone1: (0.0, 0.0, 0.0),
        }
    }
}

impl<'a> From<&'a Body> for SkeletonAttr {
    fn from(body: &'a Body) -> Self {
        use comp::object::Body::*;
        Self {
            bone0: match body {
                Crossbow => (0.0, 0.0, 14.0),
                _ => (0.0, 0.0, 0.0),
            },
            bone1: match body {
                Crossbow => (0.0, 0.0, 8.0),
                _ => (0.0, 0.0, 0.0),
            },
        }
    }
}
