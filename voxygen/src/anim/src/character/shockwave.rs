use super::{
    super::{vek::*, Animation},
    CharacterSkeleton, SkeletonAttr,
};
use common::{comp::item::ToolKind, states::utils::StageSection};
use std::f32::consts::PI;

pub struct Input {
    pub attack: bool,
}
pub struct ShockwaveAnimation;

impl Animation for ShockwaveAnimation {
    type Dependency = (
        Option<ToolKind>,
        Option<ToolKind>,
        f64,
        f32,
        Option<StageSection>,
    );
    type Skeleton = CharacterSkeleton;

    #[cfg(feature = "use-dyn-lib")]
    const UPDATE_FN: &'static [u8] = b"character_shockwave\0";

    #[cfg_attr(feature = "be-dyn-lib", export_name = "character_shockwave")]
    #[allow(clippy::single_match)] // TODO: Pending review in #587
    fn update_skeleton_inner(
        skeleton: &Self::Skeleton,
        (_active_tool_kind, _second_tool_kind, _global_time, velocity, stage_section): Self::Dependency,
        anim_time: f64,
        rate: &mut f32,
        s_a: &SkeletonAttr,
    ) -> Self::Skeleton {
        *rate = 1.0;
        let mut next = (*skeleton).clone();

        let (movement1, movement2, movement3) = match stage_section {
            Some(StageSection::Buildup) => (anim_time as f32, 0.0, 0.0),
            Some(StageSection::Swing) => (1.0, anim_time as f32, 0.0),
            Some(StageSection::Recover) => (1.0, 1.0, anim_time as f32),
            _ => (0.0, 0.0, 0.0),
        };

        next.head.position = Vec3::new(0.0, s_a.head.0, s_a.head.1);

        next.hand_l.position = Vec3::new(0.0, 0.0, -4.0);
        next.hand_l.orientation = Quaternion::rotation_x(1.27) * Quaternion::rotation_y(0.0);
        next.hand_r.position = Vec3::new(0.0, 0.0, 2.0);
        next.hand_r.orientation = Quaternion::rotation_x(1.57) * Quaternion::rotation_y(0.2);
        next.main.position = Vec3::new(0.0, 0.0, 13.2);
        next.main.orientation = Quaternion::rotation_y(PI);

        next.control.position = Vec3::new(-4.0, 7.0, 4.0);
        next.control.orientation = Quaternion::rotation_x(-0.3) * Quaternion::rotation_y(0.15);

        let twist = movement1 * 0.8;

        next.control.position = Vec3::new(
            -4.0 + movement1 * 5.0 + movement3 * -5.0,
            7.0 + movement1 * 3.0 + movement3 * -3.0,
            4.0 + movement1 * 10.0 + movement2 * -2.0 + movement3 * -8.0,
        );
        next.control.orientation =
            Quaternion::rotation_x(-0.3 + movement1 * 0.8 + movement2 * 0.3 + movement3 * -1.1)
                * Quaternion::rotation_y(
                    0.15 + movement1 * -0.15 + movement2 * 0.3 + movement3 * -0.45,
                )
                * Quaternion::rotation_z(movement1 * 0.8 + movement2 * -0.8);

        next.head.orientation = Quaternion::rotation_x(movement1 * 0.4 + movement3 * -0.4)
            * Quaternion::rotation_z(twist * 0.2 + movement2 * -0.8 + movement3 * 0.6);

        next.chest.position = Vec3::new(
            0.0,
            s_a.chest.0,
            s_a.chest.1 + movement1 * 2.0 + movement2 * -4.0 + movement3 * 2.0,
        );
        next.chest.orientation = Quaternion::rotation_x(movement2 * -0.8 + movement3 * 0.8)
            * Quaternion::rotation_z(twist * -0.2 + movement2 * -0.1 + movement3 * 0.3);

        next.belt.orientation = Quaternion::rotation_x(movement2 * 0.2 + movement3 * -0.2)
            * Quaternion::rotation_z(twist * 0.6 + movement2 * -0.48);

        next.shorts.orientation = Quaternion::rotation_x(movement2 * 0.3 + movement3 * -0.3)
            * Quaternion::rotation_z(twist + movement2 * -0.8);

        if velocity < 0.5 {
            next.foot_l.position = Vec3::new(
                -s_a.foot.0,
                s_a.foot.1 + movement1 * -7.0 + movement2 * 7.0,
                s_a.foot.2,
            );
            next.foot_l.orientation = Quaternion::rotation_x(movement1 * -0.8 + movement2 * 0.8)
                * Quaternion::rotation_z(movement1 * 0.3 + movement2 * -0.3);

            next.foot_r.position = Vec3::new(
                s_a.foot.0,
                s_a.foot.1 + movement1 * 5.0 + movement2 * -5.0,
                s_a.foot.2,
            );
            next.foot_r.orientation = Quaternion::rotation_y(movement1 * -0.3 + movement2 * 0.3)
                * Quaternion::rotation_z(movement1 * 0.4 + movement2 * -0.4);
        }
        next
    }
}
