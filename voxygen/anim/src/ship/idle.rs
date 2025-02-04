use super::{
    super::{vek::*, Animation},
    ShipSkeleton, SkeletonAttr,
};
use common::comp::item::ToolKind;

pub struct IdleAnimation;

impl Animation for IdleAnimation {
    type Dependency = (Option<ToolKind>, Option<ToolKind>, f32);
    type Skeleton = ShipSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"ship_idle\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "ship_idle")]
    #[allow(clippy::approx_constant)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_active_tool_kind, _second_tool_kind, _global_time): Self::Dependency,
        _anim_time: f32,
        _rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        let mut next = (*skeleton).clone();

        next.bone0.position = Vec3::new(s_a.bone0.0, s_a.bone0.1, s_a.bone0.2) / 11.0;

        next.bone1.position = Vec3::new(s_a.bone1.0, s_a.bone1.1, s_a.bone1.2) / 11.0;

        next
    }
}
