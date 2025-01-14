use crate::ui::img_ids::{BlankGraphic, ImageGraphic, VoxelPixArtGraphic};

// TODO: Combine with image_ids, see macro definition
rotation_image_ids! {
    pub struct ImgsRot {
        <VoxelGraphic>

        <ImageGraphic>
        indicator_mmap_small: "voxygen.element.buttons.indicator_mmap",
         // Tooltip Test
         tt_side: "voxygen.element.frames.tt_test_edge",
         tt_corner: "voxygen.element.frames.tt_test_corner_tr",
//////////////////////////////////////////////////////////////////////////////////////////////////////

        <VoxelPixArtGraphic>
    }
}

image_ids! {
    pub struct Imgs {
        <VoxelGraphic>
        // Missing: Buff Frame Animation .gif ?! we could do animation in ui.maintain, or in shader?
////////////////////////////////////////////////////////////////////////
        <VoxelPixArtGraphic>
        // Icons
        flower: "voxygen.element.icons.item_flower",
        grass: "voxygen.element.icons.item_grass",

        // Items
        potion_red: "voxygen.voxel.object.potion_red",
        potion_green: "voxygen.voxel.object.potion_green",
        potion_blue: "voxygen.voxel.object.potion_blue",
        key: "voxygen.voxel.object.key",
        key_gold: "voxygen.voxel.object.key_gold",


//////////////////////////////////////////////////////////////////////////////////////////////////////

        <ImageGraphic>

        // Checkboxes and Radio buttons
        check: "voxygen.element.buttons.radio.inactive",
        check_mo: "voxygen.element.buttons.radio.inactive_hover",
        check_press: "voxygen.element.buttons.radio.press",
        check_checked: "voxygen.element.buttons.radio.active",
        check_checked_mo: "voxygen.element.buttons.radio.hover",
        checkbox: "voxygen.element.buttons.checkbox.inactive",
        checkbox_mo: "voxygen.element.buttons.checkbox.inactive_hover",
        checkbox_press: "voxygen.element.buttons.checkbox.press",
        checkbox_checked: "voxygen.element.buttons.checkbox.active",
        checkbox_checked_mo: "voxygen.element.buttons.checkbox.hover",

        // Selection Frame
        selection: "voxygen.element.frames.selection",
        selection_hover: "voxygen.element.frames.selection_hover",
        selection_press: "voxygen.element.frames.selection_press",

        // Prompt Dialog
        prompt_top: "voxygen.element.frames.prompt_dialog_top",
        prompt_mid: "voxygen.element.frames.prompt_dialog_mid",
        prompt_bot: "voxygen.element.frames.prompt_dialog_bot",
        key_button: "voxygen.element.buttons.key_button",
        key_button_press: "voxygen.element.buttons.key_button_press",

        // Diary Window
        diary_bg: "voxygen.element.misc_bg.diary_bg",
        diary_frame: "voxygen.element.misc_bg.diary_frame",
        diary_exp_bg: "voxygen.element.misc_bg.diary_exp_bg",
        diary_exp_frame: "voxygen.element.misc_bg.diary_exp_frame",

        // Skill Trees
        slot_skills: "voxygen.element.buttons.slot_skilltree",
        swords_crossed: "voxygen.element.icons.swords_crossed",
        sceptre: "voxygen.element.icons.sceptre",
        sword: "voxygen.element.icons.sword",
        axe: "voxygen.element.icons.axe",
        hammer: "voxygen.element.icons.hammer",
        bow: "voxygen.element.icons.bow",
        staff: "voxygen.element.icons.staff",
        lock: "voxygen.element.icons.lock",
        wpn_icon_border_skills: "voxygen.element.buttons.border_skills",
        wpn_icon_border: "voxygen.element.buttons.border",
        wpn_icon_border_mo: "voxygen.element.buttons.border_mo",
        wpn_icon_border_press: "voxygen.element.buttons.border_press",
        wpn_icon_border_pressed: "voxygen.element.buttons.border_pressed",

        // Social Window
        social_frame_on: "voxygen.element.misc_bg.social_frame",
        social_bg_on: "voxygen.element.misc_bg.social_bg",
        social_frame_friends: "voxygen.element.misc_bg.social_frame",
        social_bg_friends: "voxygen.element.misc_bg.social_bg",
        social_frame_fact: "voxygen.element.misc_bg.social_frame",
        social_bg_fact: "voxygen.element.misc_bg.social_bg",
        social_tab_act: "voxygen.element.buttons.social_tab_active",
        social_tab_online: "voxygen.element.misc_bg.social_tab_online",
        social_tab_inact: "voxygen.element.buttons.social_tab_inactive",
        social_tab_inact_hover: "voxygen.element.buttons.social_tab_inactive",
        social_tab_inact_press: "voxygen.element.buttons.social_tab_inactive",

        // Crafting Window
        crafting_window: "voxygen.element.misc_bg.crafting",
        crafting_frame: "voxygen.element.misc_bg.crafting_frame",
        crafting_icon_bordered: "voxygen.element.icons.anvil",
        crafting_icon: "voxygen.element.buttons.anvil",
        crafting_icon_hover: "voxygen.element.buttons.anvil_hover",
        crafting_icon_press: "voxygen.element.buttons.anvil_press",

        // Group Window
        member_frame: "voxygen.element.frames.group_member_frame",
        member_bg: "voxygen.element.frames.group_member_bg",

        // Chat-Arrows
        chat_arrow: "voxygen.element.buttons.arrow_down",
        chat_arrow_mo: "voxygen.element.buttons.arrow_down_hover",
        chat_arrow_press: "voxygen.element.buttons.arrow_down_press",

        // Settings Window
        settings_button: "voxygen.element.buttons.settings_button",
        settings_button_pressed: "voxygen.element.buttons.settings_button_pressed",
        settings_button_hover: "voxygen.element.buttons.settings_button_hover",
        settings_button_press: "voxygen.element.buttons.settings_button_press",

        quest_bg: "voxygen.element.misc_bg.temp_quest_bg",

        // Slider
        slider: "voxygen.element.slider.track",
        slider_indicator: "voxygen.element.slider.indicator",
        slider_indicator_small: "voxygen.element.slider.indicator_round",

        // Buttons
        settings: "voxygen.element.buttons.settings",
        settings_hover: "voxygen.element.buttons.settings_hover",
        settings_press: "voxygen.element.buttons.settings_press",

        social: "voxygen.element.buttons.social",
        social_hover: "voxygen.element.buttons.social_hover",
        social_press: "voxygen.element.buttons.social_press",

        map_button: "voxygen.element.buttons.map",
        map_hover: "voxygen.element.buttons.map_hover",
        map_press: "voxygen.element.buttons.map_press",

        spellbook_button: "voxygen.element.buttons.spellbook",
        spellbook_hover: "voxygen.element.buttons.spellbook_hover",
        spellbook_press: "voxygen.element.buttons.spellbook_press",

        group_icon: "voxygen.element.buttons.group",
        group_icon_hover: "voxygen.element.buttons.group_hover",
        group_icon_press: "voxygen.element.buttons.group_press",

        sp_indicator_arrow: "voxygen.element.buttons.arrow_down_gold",

        // Skill Icons
        twohsword_m1: "voxygen.element.icons.2hsword_m1",
        twohsword_m2: "voxygen.element.icons.2hsword_m2",
        onehdagger_m1: "voxygen.element.icons.daggers",
        onehdagger_m2: "voxygen.element.icons.skill_slice_2",
        onehshield_m1: "voxygen.element.icons.swordshield",
        onehshield_m2: "voxygen.element.icons.character",
        twohhammer_m1: "voxygen.element.icons.2hhammer_m1",
        twohaxe_m1: "voxygen.element.icons.2haxe_m1",
        bow_m1: "voxygen.element.icons.bow_m1",
        bow_m2: "voxygen.element.icons.bow_m2",
        staff_melee: "voxygen.element.icons.staff_m1",
        fireball: "voxygen.element.icons.staff_m2",
        flyingrod_m1: "voxygen.element.icons.debug_wand_m1",
        flyingrod_m2: "voxygen.element.icons.debug_wand_m2",
        sword_pierce: "voxygen.element.icons.skill_sword_pierce",
        hammergolf: "voxygen.element.icons.skill_hammergolf",
        axespin: "voxygen.element.icons.skill_axespin",
        fire_aoe: "voxygen.element.icons.skill_fire_aoe",
        flamethrower: "voxygen.element.icons.skill_flamethrower",

        // Skilltree Icons
        health_plus_skill: "voxygen.element.icons.skilltree.health_plus",
        stamina_plus_skill: "voxygen.element.icons.skilltree.stamina_plus",
        unlock_axe_skill: "voxygen.element.icons.skilltree.unlock_axe",
        unlock_bow_skill: "voxygen.element.icons.skilltree.unlock_bow",
        unlock_hammer_skill: "voxygen.element.icons.skilltree.unlock_hammer",
        unlock_sceptre_skill: "voxygen.element.icons.skilltree.unlock_sceptre",
        unlock_staff_skill0: "voxygen.element.icons.skilltree.unlock_staff-0",
        unlock_sword_skill: "voxygen.element.icons.skilltree.unlock_sword",
        skill_dodge_skill: "voxygen.element.icons.skilltree.skill_dodge",

        buff_amount_skill: "voxygen.element.icons.skilltree.buff_amount",
        buff_combo_skill: "voxygen.element.icons.skilltree.buff_combo",
        buff_cost_skill: "voxygen.element.icons.skilltree.buff_cost",
        buff_damage_skill: "voxygen.element.icons.skilltree.buff_damage",
        buff_distance_skill: "voxygen.element.icons.skilltree.buff_distance",
        buff_energy_drain_skill: "voxygen.element.icons.skilltree.buff_energy_drain",
        buff_energy_regen_skill: "voxygen.element.icons.skilltree.buff_energy_regen",
        buff_explosion_skill: "voxygen.element.icons.skilltree.buff_explosion",
        buff_heal_skill: "voxygen.element.icons.skilltree.buff_heal",
        buff_helicopter_skill: "voxygen.element.icons.skilltree.buff_helicopter",
        buff_infinite_skill: "voxygen.element.icons.skilltree.buff_infinite",
        buff_knockback_skill: "voxygen.element.icons.skilltree.buff_knockback",
        buff_lifesteal_skill: "voxygen.element.icons.skilltree.buff_lifesteal",
        buff_projectile_speed_skill: "voxygen.element.icons.skilltree.buff_projectile_speed",
        buff_radius_skill: "voxygen.element.icons.skilltree.buff_radius",
        buff_speed_skill: "voxygen.element.icons.skilltree.buff_speed",

        debuff_amount_skill: "voxygen.element.icons.skilltree.debuff_amount",
        debuff_combo_skill: "voxygen.element.icons.skilltree.debuff_combo",
        debuff_cost_skill: "voxygen.element.icons.skilltree.debuff_cost",
        debuff_damage_skill: "voxygen.element.icons.skilltree.debuff_damage",
        debuff_distance_skill: "voxygen.element.icons.skilltree.debuff_distance",
        debuff_energy_drain_skill: "voxygen.element.icons.skilltree.debuff_energy_drain",
        debuff_energy_regen_skill: "voxygen.element.icons.skilltree.debuff_energy_regen",
        debuff_explosion_skill: "voxygen.element.icons.skilltree.debuff_explosion",
        debuff_heal_skill: "voxygen.element.icons.skilltree.debuff_heal",
        debuff_helicopter_skill: "voxygen.element.icons.skilltree.debuff_helicopter",
        debuff_infinite_skill: "voxygen.element.icons.skilltree.debuff_infinite",
        debuff_knockback_skill: "voxygen.element.icons.skilltree.debuff_knockback",
        debuff_lifesteal_skill: "voxygen.element.icons.skilltree.debuff_lifesteal",
        debuff_projectile_speed_skill: "voxygen.element.icons.skilltree.debuff_projectile_speed",
        debuff_radius_skill: "voxygen.element.icons.skilltree.debuff_radius",
        debuff_speed_skill: "voxygen.element.icons.skilltree.debuff_speed",

        heal_amount_skill: "voxygen.element.icons.skilltree.heal_amount",
        heal_combo_skill: "voxygen.element.icons.skilltree.heal_combo",
        heal_cost_skill: "voxygen.element.icons.skilltree.heal_cost",
        heal_damage_skill: "voxygen.element.icons.skilltree.heal_damage",
        heal_distance_skill: "voxygen.element.icons.skilltree.heal_distance",
        heal_energy_drain_skill: "voxygen.element.icons.skilltree.heal_energy_drain",
        heal_energy_regen_skill: "voxygen.element.icons.skilltree.heal_energy_regen",
        heal_explosion_skill: "voxygen.element.icons.skilltree.heal_explosion",
        heal_heal_skill: "voxygen.element.icons.skilltree.heal_heal",
        heal_helicopter_skill: "voxygen.element.icons.skilltree.heal_helicopter",
        heal_infinite_skill: "voxygen.element.icons.skilltree.heal_infinite",
        heal_knockback_skill: "voxygen.element.icons.skilltree.heal_knockback",
        heal_lifesteal_skill: "voxygen.element.icons.skilltree.heal_lifesteal",
        heal_projectile_speed_skill: "voxygen.element.icons.skilltree.heal_projectile_speed",
        heal_radius_skill: "voxygen.element.icons.skilltree.heal_radius",
        heal_speed_skill: "voxygen.element.icons.skilltree.heal_speed",

        magic_amount_skill: "voxygen.element.icons.skilltree.magic_amount",
        magic_combo_skill: "voxygen.element.icons.skilltree.magic_combo",
        magic_cost_skill: "voxygen.element.icons.skilltree.magic_cost",
        magic_damage_skill: "voxygen.element.icons.skilltree.magic_damage",
        magic_distance_skill: "voxygen.element.icons.skilltree.magic_distance",
        magic_energy_drain_skill: "voxygen.element.icons.skilltree.magic_energy_drain",
        magic_energy_regen_skill: "voxygen.element.icons.skilltree.magic_energy_regen",
        magic_explosion_skill: "voxygen.element.icons.skilltree.magic_explosion",
        magic_heal_skill: "voxygen.element.icons.skilltree.magic_heal",
        magic_helicopter_skill: "voxygen.element.icons.skilltree.magic_helicopter",
        magic_infinite_skill: "voxygen.element.icons.skilltree.magic_infinite",
        magic_knockback_skill: "voxygen.element.icons.skilltree.magic_knockback",
        magic_lifesteal_skill: "voxygen.element.icons.skilltree.magic_lifesteal",
        magic_projectile_speed_skill: "voxygen.element.icons.skilltree.magic_projectile_speed",
        magic_radius_skill: "voxygen.element.icons.skilltree.magic_radius",
        magic_speed_skill: "voxygen.element.icons.skilltree.magic_speed",

        physical_amount_skill: "voxygen.element.icons.skilltree.physical_amount",
        physical_combo_skill: "voxygen.element.icons.skilltree.physical_combo",
        physical_cost_skill: "voxygen.element.icons.skilltree.physical_cost",
        physical_damage_skill: "voxygen.element.icons.skilltree.physical_damage",
        physical_distance_skill: "voxygen.element.icons.skilltree.physical_distance",
        physical_energy_drain_skill: "voxygen.element.icons.skilltree.physical_energy_drain",
        physical_energy_regen_skill: "voxygen.element.icons.skilltree.physical_energy_regen",
        physical_explosion_skill: "voxygen.element.icons.skilltree.physical_explosion",
        physical_heal_skill: "voxygen.element.icons.skilltree.physical_heal",
        physical_helicopter_skill: "voxygen.element.icons.skilltree.physical_helicopter",
        physical_infinite_skill: "voxygen.element.icons.skilltree.physical_infinite",
        physical_knockback_skill: "voxygen.element.icons.skilltree.physical_knockback",
        physical_lifesteal_skill: "voxygen.element.icons.skilltree.physical_lifesteal",
        physical_projectile_speed_skill: "voxygen.element.icons.skilltree.physical_projectile_speed",
        physical_radius_skill: "voxygen.element.icons.skilltree.physical_radius",
        physical_speed_skill: "voxygen.element.icons.skilltree.physical_speed",

        utility_amount_skill: "voxygen.element.icons.skilltree.utility_amount",
        utility_combo_skill: "voxygen.element.icons.skilltree.utility_combo",
        utility_cost_skill: "voxygen.element.icons.skilltree.utility_cost",
        utility_damage_skill: "voxygen.element.icons.skilltree.utility_damage",
        utility_distance_skill: "voxygen.element.icons.skilltree.utility_distance",
        utility_energy_drain_skill: "voxygen.element.icons.skilltree.utility_energy_drain",
        utility_energy_regen_skill: "voxygen.element.icons.skilltree.utility_energy_regen",
        utility_explosion_skill: "voxygen.element.icons.skilltree.utility_explosion",
        utility_heal_skill: "voxygen.element.icons.skilltree.utility_heal",
        utility_helicopter_skill: "voxygen.element.icons.skilltree.utility_helicopter",
        utility_infinite_skill: "voxygen.element.icons.skilltree.utility_infinite",
        utility_knockback_skill: "voxygen.element.icons.skilltree.utility_knockback",
        utility_lifesteal_skill: "voxygen.element.icons.skilltree.utility_lifesteal",
        utility_projectile_speed_skill: "voxygen.element.icons.skilltree.utility_projectile_speed",
        utility_radius_skill: "voxygen.element.icons.skilltree.utility_radius",
        utility_speed_skill: "voxygen.element.icons.skilltree.utility_speed",

        // Skillbar
        level_up: "voxygen.element.misc_bg.level_up",
        bar_content: "voxygen.element.skillbar.bar_content",
        skillbar_bg: "voxygen.element.skillbar.bg",
        skillbar_frame: "voxygen.element.skillbar.frame",
        health_bg: "voxygen.element.skillbar.health_bg",
        health_frame: "voxygen.element.skillbar.health_frame",
        stamina_bg: "voxygen.element.skillbar.stamina_bg",
        stamina_frame: "voxygen.element.skillbar.stamina_frame",
        m1_ico: "voxygen.element.icons.m1",
        m2_ico: "voxygen.element.icons.m2",
        m_scroll_ico: "voxygen.element.icons.m_scroll",
        m_move_ico: "voxygen.element.icons.m_move",
        skillbar_slot: "voxygen.element.skillbar.slot",

        // Other Icons/Art
        skull: "voxygen.element.icons.skull",
        skull_2: "voxygen.element.icons.skull_2",
        fireplace: "voxygen.element.misc_bg.fireplace",

        // Crosshair
        crosshair_inner: "voxygen.element.misc_bg.crosshair_inner",

        crosshair_outer_round: "voxygen.element.misc_bg.crosshair_outer_1",
        crosshair_outer_round_edges: "voxygen.element.misc_bg.crosshair_outer_2",
        crosshair_outer_edges: "voxygen.element.misc_bg.crosshair_outer_3",

        crosshair_bg: "voxygen.element.misc_bg.crosshair_bg",
        crosshair_bg_hover: "voxygen.element.misc_bg.crosshair_bg_hover",
        crosshair_bg_press: "voxygen.element.misc_bg.crosshair_bg_press",
        crosshair_bg_pressed: "voxygen.element.misc_bg.crosshair_bg_pressed",

        // Map
        map_bg: "voxygen.element.misc_bg.map_bg",
        map_frame: "voxygen.element.misc_bg.map_frame",
        map_frame_art: "voxygen.element.misc_bg.map_frame_art",
        indicator_mmap: "voxygen.element.buttons.indicator_mmap",
        indicator_map_overlay: "voxygen.element.buttons.indicator_mmap_small",
        indicator_group: "voxygen.element.map.group_indicator",
        indicator_group_up: "voxygen.element.map.group_indicator_arrow_up",
        indicator_group_down: "voxygen.element.map.group_indicator_arrow_down",

        // MiniMap
        mmap_frame: "voxygen.element.frames.mmap",
        mmap_frame_2: "voxygen.element.frames.mmap_frame",
        mmap_frame_closed: "voxygen.element.frames.mmap_closed",
        mmap_closed: "voxygen.element.buttons.button_mmap_closed",
        mmap_closed_hover: "voxygen.element.buttons.button_mmap_closed_hover",
        mmap_closed_press: "voxygen.element.buttons.button_mmap_closed_press",
        mmap_open: "voxygen.element.buttons.button_mmap_open",
        mmap_open_hover: "voxygen.element.buttons.button_mmap_open_hover",
        mmap_open_press: "voxygen.element.buttons.button_mmap_open_press",
        mmap_plus: "voxygen.element.buttons.min_plus.mmap_button-plus",
        mmap_plus_hover: "voxygen.element.buttons.min_plus.mmap_button-plus_hover",
        mmap_plus_press: "voxygen.element.buttons.min_plus.mmap_button-plus_press",
        mmap_minus: "voxygen.element.buttons.min_plus.mmap_button-min",
        mmap_minus_hover: "voxygen.element.buttons.min_plus.mmap_button-min_hover",
        mmap_minus_press: "voxygen.element.buttons.min_plus.mmap_button-min_press",
        mmap_north: "voxygen.element.buttons.min_plus.mmap_button-north",
        mmap_north_hover: "voxygen.element.buttons.min_plus.mmap_button-north_hover",
        mmap_north_press: "voxygen.element.buttons.min_plus.mmap_button-north_press",
        mmap_north_press_hover: "voxygen.element.buttons.min_plus.mmap_button-north_press_hover",
        map_dif_1: "voxygen.element.map.dif_1",
        map_dif_2: "voxygen.element.map.dif_2",
        map_dif_3: "voxygen.element.map.dif_3",
        map_dif_4: "voxygen.element.map.dif_4",
        map_dif_5: "voxygen.element.map.dif_5",
        map_dif_6: "voxygen.element.map.dif_6",
        mmap_site_town: "voxygen.element.map.town",
        mmap_site_town_hover: "voxygen.element.map.town_hover",
        mmap_site_town_bg: "voxygen.element.map.town_bg",
        mmap_site_dungeon: "voxygen.element.map.dungeon",
        mmap_site_dungeon_hover: "voxygen.element.map.dungeon_hover",
        mmap_site_dungeon_bg: "voxygen.element.map.dungeon_bg",
        mmap_site_castle: "voxygen.element.map.castle",
        mmap_site_castle_hover: "voxygen.element.map.castle_hover",
        mmap_site_castle_bg: "voxygen.element.map.castle_bg",
        mmap_site_cave_bg: "voxygen.element.map.cave_bg",
        mmap_site_cave_hover: "voxygen.element.map.cave_hover",
        mmap_site_cave: "voxygen.element.map.cave",
        mmap_site_excl: "voxygen.element.map.excl",
        mmap_site_tree: "voxygen.element.map.tree",
        mmap_site_tree_hover: "voxygen.element.map.tree_hover",

        // Window Parts
        window_3: "voxygen.element.frames.window_3",
        esc_frame: "voxygen.element.frames.esc_menu",
        // Settings
        settings_bg: "voxygen.element.misc_bg.settings_bg",
        settings_frame: "voxygen.element.misc_bg.settings_frame",

        // Close-Button
        close_btn: "voxygen.element.buttons.close_btn",
        close_btn_hover: "voxygen.element.buttons.close_btn_hover",
        close_btn_press: "voxygen.element.buttons.close_btn_press",
        close_button: "voxygen.element.buttons.close_btn",
        close_button_hover: "voxygen.element.buttons.close_btn_hover",
        close_button_press: "voxygen.element.buttons.close_btn_press",

        // Inventory
        collapse_btn: "voxygen.element.buttons.inv_collapse",
        collapse_btn_hover: "voxygen.element.buttons.inv_collapse_hover",
        collapse_btn_press: "voxygen.element.buttons.inv_collapse_press",
        expand_btn: "voxygen.element.buttons.inv_expand",
        expand_btn_hover: "voxygen.element.buttons.inv_expand_hover",
        expand_btn_press: "voxygen.element.buttons.inv_expand_press",
        coin_ico: "voxygen.element.icons.coin",
        inv_bg_armor: "voxygen.element.misc_bg.inv_bg_0",
        inv_bg_stats: "voxygen.element.misc_bg.inv_bg_1",
        inv_frame: "voxygen.element.misc_bg.inv_frame",
        inv_frame_bag: "voxygen.element.misc_bg.inv_frame_bag",
        inv_bg_bag: "voxygen.element.misc_bg.inv_bg_bag",
        char_art: "voxygen.element.icons.character",
        inv_slot: "voxygen.element.buttons.inv_slot",
        inv_slot_grey: "voxygen.element.buttons.inv_slot_grey",
        inv_slot_green: "voxygen.element.buttons.inv_slot_green",
        inv_slot_blue: "voxygen.element.buttons.inv_slot_blue",
        inv_slot_purple: "voxygen.element.buttons.inv_slot_purple",
        inv_slot_gold: "voxygen.element.buttons.inv_slot_gold",
        inv_slot_orange: "voxygen.element.buttons.inv_slot_orange",
        inv_slot_red: "voxygen.element.buttons.inv_slot_red",
        inv_slot_sel: "voxygen.element.buttons.inv_slot_sel",
        scrollbar_bg: "voxygen.element.slider.scrollbar",
        scrollbar_bg_big: "voxygen.element.slider.scrollbar_1",
        inv_tab_active: "voxygen.element.buttons.inv_tab_active",
        inv_tab_inactive: "voxygen.element.buttons.inv_tab_inactive",
        inv_tab_inactive_hover: "voxygen.element.buttons.inv_tab_inactive",
        inv_tab_inactive_press: "voxygen.element.buttons.inv_tab_inactive",
        armor_slot: "voxygen.element.buttons.armor_slot",
        armor_slot_sel: "voxygen.element.buttons.armor_slot_selected",
        armor_slot_empty: "voxygen.element.buttons.armor_slot_empty",
        head_bg: "voxygen.element.icons.head",
        shoulders_bg: "voxygen.element.icons.shoulders",
        hands_bg: "voxygen.element.icons.hands",
        belt_bg: "voxygen.element.icons.belt",
        legs_bg: "voxygen.element.icons.legs",
        feet_bg: "voxygen.element.icons.feet",
        ring_bg: "voxygen.element.icons.ring",
        tabard_bg: "voxygen.element.icons.tabard",
        glider_bg: "voxygen.element.icons.glider",
        chest_bg: "voxygen.element.icons.chest",
        back_bg: "voxygen.element.icons.back",
        lantern_bg: "voxygen.element.icons.lantern",
        necklace_bg: "voxygen.element.icons.necklace",
        mainhand_bg: "voxygen.element.icons.mainhand",
        bag_bg: "voxygen.element.icons.bag",
        offhand_bg: "voxygen.element.icons.offhand",
        stamina_ico: "voxygen.element.icons.stamina",
        health_ico: "voxygen.element.icons.health",
        protection_ico: "voxygen.element.icons.protection",
        combat_rating_ico: "voxygen.element.icons.combat_rating",
        combat_rating_ico_shadow: "voxygen.element.icons.combat_rating_shadow",

        not_found: "voxygen.element.not_found",

        death_bg: "voxygen.background.death",
        hurt_bg: "voxygen.background.hurt",

        banner_top: "voxygen.element.frames.banner_top",

        // Icons
        snake_arrow_0: "voxygen.element.icons.snake",
        skill_sceptre_lifesteal: "voxygen.element.icons.lifesteal",
        sword_whirlwind: "voxygen.element.icons.sword_whirlwind",
        skill_sceptre_heal: "voxygen.element.icons.heal_0",
        hammerleap: "voxygen.element.icons.skill_hammerleap",
        skill_axe_leap_slash: "voxygen.element.icons.skill_axe_leap_slash",
        skill_bow_jump_burst: "voxygen.element.icons.skill_bow_jump_burst",
        skill_sceptre_aura: "voxygen.element.icons.sceptre_protection",
        missing_icon: "voxygen.element.icons.missing_icon_grey",

        // Buttons
        button: "voxygen.element.buttons.button",
        button_hover: "voxygen.element.buttons.button_hover",
        button_press: "voxygen.element.buttons.button_press",

        // Enemy Healthbar
        enemy_health: "voxygen.element.frames.enemybar",
        enemy_health_bg: "voxygen.element.frames.enemybar_bg",
        health_bar_group: "voxygen.element.frames.enemybar_1",
        health_bar_group_bg: "voxygen.element.frames.enemybar_bg_1",
        // Enemy Bar Content:
        enemy_bar: "voxygen.element.skillbar.enemy_bar_content",

        // Bag
        bag: "voxygen.element.buttons.bag.closed",
        bag_hover: "voxygen.element.buttons.bag.closed_hover",
        bag_press: "voxygen.element.buttons.bag.closed_press",
        bag_open: "voxygen.element.buttons.bag.open",
        bag_open_hover: "voxygen.element.buttons.bag.open_hover",
        bag_open_press: "voxygen.element.buttons.bag.open_press",

        map_icon: "voxygen.element.icons.map",

        grid_button: "voxygen.element.buttons.border",
        grid_button_hover: "voxygen.element.buttons.border_mo",
        grid_button_press: "voxygen.element.buttons.border_press",
        grid_button_open: "voxygen.element.buttons.border_pressed",

        // Char Window
        progress_frame: "voxygen.element.frames.progress_bar",
        progress: "voxygen.element.misc_bg.progress",

        // Speech bubbles
        speech_bubble_top_left: "voxygen.element.frames.bubble.top_left",
        speech_bubble_top: "voxygen.element.frames.bubble.top",
        speech_bubble_top_right: "voxygen.element.frames.bubble.top_right",
        speech_bubble_left: "voxygen.element.frames.bubble.left",
        speech_bubble_mid: "voxygen.element.frames.bubble.mid",
        speech_bubble_right: "voxygen.element.frames.bubble.right",
        speech_bubble_bottom_left: "voxygen.element.frames.bubble.bottom_left",
        speech_bubble_bottom: "voxygen.element.frames.bubble.bottom",
        speech_bubble_bottom_right: "voxygen.element.frames.bubble.bottom_right",
        speech_bubble_tail: "voxygen.element.frames.bubble.tail",
        speech_bubble_icon_frame: "voxygen.element.frames.bubble_dark.icon_frame",

        dark_bubble_top_left: "voxygen.element.frames.bubble_dark.top_left",
        dark_bubble_top: "voxygen.element.frames.bubble_dark.top",
        dark_bubble_top_right: "voxygen.element.frames.bubble_dark.top_right",
        dark_bubble_left: "voxygen.element.frames.bubble_dark.left",
        dark_bubble_mid: "voxygen.element.frames.bubble_dark.mid",
        dark_bubble_right: "voxygen.element.frames.bubble_dark.right",
        dark_bubble_bottom_left: "voxygen.element.frames.bubble_dark.bottom_left",
        dark_bubble_bottom: "voxygen.element.frames.bubble_dark.bottom",
        dark_bubble_bottom_right: "voxygen.element.frames.bubble_dark.bottom_right",
        dark_bubble_tail: "voxygen.element.frames.bubble_dark.tail",
        dark_bubble_icon_frame: "voxygen.element.frames.bubble_dark.icon_frame",


        // Chat icons
        chat_faction_small: "voxygen.element.icons.chat.faction_small",
        chat_group_small: "voxygen.element.icons.chat.group_small",
        chat_kill_small: "voxygen.element.icons.chat.kill_small",
        chat_region_small: "voxygen.element.icons.chat.region_small",
        chat_say_small: "voxygen.element.icons.chat.say_small",
        chat_tell_small: "voxygen.element.icons.chat.tell_small",
        chat_world_small: "voxygen.element.icons.chat.world_small",
        chat_command_error_small: "voxygen.element.icons.chat.command_error_small",
        chat_command_info_small: "voxygen.element.icons.chat.command_info_small",
        chat_online_small: "voxygen.element.icons.chat.online_small",
        chat_offline_small: "voxygen.element.icons.chat.offline_small",
        chat_loot_small: "voxygen.element.icons.chat.loot_small",

        chat_faction: "voxygen.element.icons.chat.faction",
        chat_group: "voxygen.element.icons.chat.group",
        chat_region: "voxygen.element.icons.chat.region",
        chat_say: "voxygen.element.icons.chat.say",
        chat_tell: "voxygen.element.icons.chat.tell",
        chat_world: "voxygen.element.icons.chat.world",

        // Buffs
        buff_plus_0: "voxygen.element.icons.de_buffs.buff_plus_0",
        buff_saturation_0: "voxygen.element.icons.de_buffs.buff_saturation_0",
        buff_potion_0: "voxygen.element.icons.de_buffs.buff_potion_0",
        buff_campfire_heal_0: "voxygen.element.icons.de_buffs.buff_campfire_heal_0",
        buff_energyplus_0: "voxygen.element.icons.de_buffs.buff_energyplus_0",
        buff_healthplus_0: "voxygen.element.icons.de_buffs.buff_healthplus_0",
        buff_invincibility_0: "voxygen.element.icons.de_buffs.buff_invincibility_0",
        buff_dmg_red_0: "voxygen.element.icons.de_buffs.buff_damage_reduce_0",

        // Debuffs
        debuff_skull_0: "voxygen.element.icons.de_buffs.debuff_skull_0",
        debuff_bleed_0: "voxygen.element.icons.de_buffs.debuff_bleed_0",

        // Animation Frames
        // Buff Frame
        buff_0: "voxygen.element.animation.buff_frame.1",
        buff_1: "voxygen.element.animation.buff_frame.2",
        buff_2: "voxygen.element.animation.buff_frame.3",
        buff_3: "voxygen.element.animation.buff_frame.4",
        buff_4: "voxygen.element.animation.buff_frame.5",
        buff_5: "voxygen.element.animation.buff_frame.6",
        buff_6: "voxygen.element.animation.buff_frame.7",
        buff_7: "voxygen.element.animation.buff_frame.8",

        <BlankGraphic>
        nothing: (),
    }
}
