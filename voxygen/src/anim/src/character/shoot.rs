use super::{
    super::{vek::*, Animation},
    CharacterSkeleton, SkeletonAttr,
};
use common::comp::item::ToolKind;

pub struct ShootAnimation;

impl Animation for ShootAnimation {
    type Dependency = (Option<ToolKind>, Option<ToolKind>, f32, f64);
    type Skeleton = CharacterSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"character_shoot\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "character_shoot")]
    #[allow(clippy::approx_constant)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (active_tool_kind, _second_tool_kind, velocity, _global_time): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;

        let mut next = (*skeleton).clone();

        let lab = 1.0;
        let foot = (((5.0)
            / (0.2 + 4.8 * ((anim_time as f32 * lab as f32 * 8.0).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 8.0).sin());
        let foote = (((5.0)
            / (0.5 + 4.5 * ((anim_time as f32 * lab as f32 * 8.0 + 1.57).sin()).powf(2.0 as f32)))
        .sqrt())
            * ((anim_time as f32 * lab as f32 * 8.0).sin());

        let exp = ((anim_time as f32).powf(0.3 as f32)).min(1.2);

        next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);
        next.head.orientation = Quaternion::rotation_z(exp * -0.4)
            * Quaternion::rotation_x(0.0)
            * Quaternion::rotation_y(exp * 0.1);

        next.chest.position = Vec3::new(0.0, s_a.chest.0 - exp * 1.5, s_a.chest.1);
        next.chest.orientation = Quaternion::rotation_z(0.4 + exp * 1.0)
            * Quaternion::rotation_x(0.0 + exp * 0.2)
            * Quaternion::rotation_y(exp * -0.08);

        next.belt.position = Vec3::new(0.0, s_a.belt.0 + exp * 1.0, s_a.belt.1);
        next.belt.orientation = next.chest.orientation * -0.1;

        next.shorts.position = Vec3::new(0.0, s_a.shorts.0 + exp * 1.0, s_a.shorts.1);
        next.shorts.orientation = next.chest.orientation * -0.08;

        match active_tool_kind {
            Some(ToolKind::Staff(_)) | Some(ToolKind::Sceptre(_)) => {
                next.hand_l.position = Vec3::new(s_a.sthl.0, s_a.sthl.1, s_a.sthl.2);
                next.hand_l.orientation = Quaternion::rotation_x(s_a.sthl.3);

                next.hand_r.position = Vec3::new(s_a.sthr.0, s_a.sthr.1, s_a.sthr.2);
                next.hand_r.orientation =
                    Quaternion::rotation_x(s_a.sthr.3) * Quaternion::rotation_y(s_a.sthr.4);

                next.main.position = Vec3::new(0.0, 0.0, 0.0);
                next.main.orientation = Quaternion::rotation_y(0.0);

                next.control.position =
                    Vec3::new(s_a.stc.0, s_a.stc.1 + exp * 5.0, s_a.stc.2 - exp * 5.0);
                next.control.orientation = Quaternion::rotation_x(s_a.stc.3 + exp * 0.4)
                    * Quaternion::rotation_y(s_a.stc.4)
                    * Quaternion::rotation_z(s_a.stc.5 + exp * 1.5);
            },
            Some(ToolKind::Bow(_)) => {
                next.hand_l.position = Vec3::new(
                    s_a.bhl.0 - exp * 2.0,
                    s_a.bhl.1 - exp * 4.0,
                    s_a.bhl.2 + exp * 6.0,
                );
                next.hand_l.orientation = Quaternion::rotation_x(s_a.bhl.3)
                    * Quaternion::rotation_y(s_a.bhl.4 + exp * 0.8)
                    * Quaternion::rotation_z(s_a.bhl.5 + exp * 0.9);
                next.hand_r.position = Vec3::new(s_a.bhr.0, s_a.bhr.1, s_a.bhr.2);
                next.hand_r.orientation = Quaternion::rotation_x(s_a.bhl.3)
                    * Quaternion::rotation_y(s_a.bhr.4)
                    * Quaternion::rotation_z(s_a.bhr.5);
                next.main.position = Vec3::new(0.0, 0.0, 0.0);
                next.main.orientation = Quaternion::rotation_x(0.0);

                next.control.position = Vec3::new(s_a.bc.0, s_a.bc.1, 4.0 + s_a.bc.2);
                next.control.orientation = Quaternion::rotation_x(s_a.bc.3 + exp * 0.4);
            },
            _ => {},
        }
        if velocity > 0.5 {
            next.foot_l.position = Vec3::new(
                -s_a.foot.0 - foot * 1.0 + exp * -1.0,
                foote * 0.8 + exp * 1.5,
                s_a.foot.2,
            );
            next.foot_l.orientation = Quaternion::rotation_x(exp * 0.5)
                * Quaternion::rotation_z(exp * 0.4)
                * Quaternion::rotation_y(0.15);

            next.foot_r.position = Vec3::new(
                s_a.foot.0 + foot * 1.0 + exp * 1.0,
                foote * -0.8 + exp * -1.0,
                s_a.foot.2,
            );
            next.foot_r.orientation = Quaternion::rotation_x(exp * -0.5)
                * Quaternion::rotation_z(exp * 0.4)
                * Quaternion::rotation_y(0.0);
            next.torso.orientation = Quaternion::rotation_x(-0.15);
        } else {
            next.foot_l.position = Vec3::new(-s_a.foot.0, -2.5, s_a.foot.2 + exp * 2.5);
            next.foot_l.orientation =
                Quaternion::rotation_x(exp * -0.2 - 0.2) * Quaternion::rotation_z(exp * 1.0);

            next.foot_r.position = Vec3::new(s_a.foot.0, 3.5 - exp * 2.0, s_a.foot.2);
            next.foot_r.orientation =
                Quaternion::rotation_x(exp * 0.1) * Quaternion::rotation_z(exp * 0.5);
        }
        next.back.orientation = Quaternion::rotation_x(-0.3);

        next.lantern.orientation =
            Quaternion::rotation_x(exp * -0.7 + 0.4) * Quaternion::rotation_y(exp * 0.4);

        next
    }
}
