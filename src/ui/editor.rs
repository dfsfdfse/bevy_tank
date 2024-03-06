use bevy::prelude::*;

use crate::{
    res::{Block, BlockState, Clear, GameMapCollection, LastSelectInfo, NodeBlock, UISelectInfo},
    utils::{
        class::StyleCommand,
        util::{is_four, vec2_to_transform_pos},
        widget::{node_children, node_root, text, GridItemInfo},
    },
};

use super::{
    class::{
        class_node_fill,
        editor_class::{
            class_node_collapse_item_default, class_node_collapse_item_hover,
            class_node_left_panel, class_node_map_name_style, class_node_map_name_text,
        },
    },
    widget::{wd_node_block, wd_setup_collapse_grid},
};

pub fn setup_ui_editor(
    commands: Commands,
    gm_maps: Res<GameMapCollection>,
    mut select_info: ResMut<UISelectInfo>,
    mut last_select_info: ResMut<LastSelectInfo>,
) {
    node_root(class_node_left_panel, commands, Clear, |gc| {
        wd_setup_collapse_grid("LEVEL", gm_maps.maps.len(), 1, 30., gc, |gc, r, c| {
            node_children(
                (
                    class_node_fill,
                    if select_info.map_editor_level_index == r {
                        class_node_collapse_item_hover
                    } else {
                        class_node_collapse_item_default
                    },
                ),
                gc,
                (Interaction::None, GridItemInfo(r, c)),
                |gc| {
                    node_children(class_node_map_name_style, gc, (), |gc| {
                        text(
                            [gm_maps.maps[r].name.as_str()],
                            class_node_map_name_text,
                            gc,
                            (),
                        );
                    });
                },
            );
        });
        wd_setup_collapse_grid("BLOCK", 5, 2, 75., gc, |gc, r, c| {
            let index = r * 2 + c;
            if index < 9 {
                //解决初始化选择时存储
                if index == select_info.map_editor_block {
                    let id = wd_node_block(
                        (Interaction::None, NodeBlock::new(0, index)),
                        gc,
                        r,
                        c,
                        select_info.as_mut(),
                    );
                    last_select_info.last_map_editor_block = Some(id);
                } else {
                    wd_node_block(
                        (Interaction::None, NodeBlock::new(0, index)),
                        gc,
                        r,
                        c,
                        select_info.as_ref(),
                    );
                }
            }
        });
    });
}

pub fn update_ui_editor(
    mut commands: Commands,
    query_event: Query<(&Interaction, &GridItemInfo), Changed<Interaction>>,
    query_entity: Query<Entity, (With<Interaction>, With<GridItemInfo>)>,
    mut ui_selector: ResMut<UISelectInfo>,
) {
    for (interaction, grid_item) in query_event.iter() {
        if *interaction == Interaction::Pressed {
            ui_selector.map_editor_level_index = grid_item.0;
        }
    }
    //优化：存储上次的选择的实体进行更新
    for (index, entity) in query_entity.iter().enumerate() {
        if index == ui_selector.map_editor_level_index {
            commands.set_style(entity, class_node_collapse_item_hover);
        } else {
            commands.set_style(entity, class_node_collapse_item_default);
        }
    }
}

pub fn update_ui_editor_brush(
    mut commands: Commands,
    mouse_event: Res<ButtonInput<MouseButton>>,
    mut ui_selector: ResMut<UISelectInfo>,
    mut last_select_info: ResMut<LastSelectInfo>,
    mut move_event: EventReader<CursorMoved>,
    mut gizmos: Gizmos,
    mut query_block: Query<(&mut Block, Entity)>,
    mut gm_maps: ResMut<GameMapCollection>,
    mut gm_panel_entity: Query<Entity, (With<Sprite>, With<Clear>)>,
) {
    for evt in move_event.read() {
        let transform_pos = vec2_to_transform_pos(evt.position);
        if transform_pos.0 > -312.
            && transform_pos.1 > -312.
            && transform_pos.1 < 312.
            && transform_pos.0 < 312.
        {
            ui_selector.map_editor_cursor = (
                ((312. - transform_pos.1) / 48.) as usize,
                ((transform_pos.0 + 312.) / 48.) as usize,
            );
            ui_selector.show_line = true;
        } else {
            ui_selector.show_line = false;
        }
    }
    if ui_selector.show_line {
        let (x_start, x_end, y_start, y_end, start_edge, end_edge) = (
            ui_selector.map_editor_cursor.1 as f32 * 48. - 312.,
            ui_selector.map_editor_cursor.1 as f32 * 48. - 264.,
            -(ui_selector.map_editor_cursor.0 as f32 * 48.) + 312.,
            -(ui_selector.map_editor_cursor.0 as f32 * 48.) + 264.,
            -312.,
            312.,
        );
        gizmos.line_2d(
            Vec2::new(x_start, start_edge),
            Vec2::new(x_start, end_edge),
            Color::RED,
        );
        gizmos.line_2d(
            Vec2::new(x_end, start_edge),
            Vec2::new(x_end, end_edge),
            Color::RED,
        );
        gizmos.line_2d(
            Vec2::new(start_edge, y_start),
            Vec2::new(end_edge, y_start),
            Color::RED,
        );
        gizmos.line_2d(
            Vec2::new(start_edge, y_end),
            Vec2::new(end_edge, y_end),
            Color::RED,
        );
        if mouse_event.pressed(MouseButton::Left) {
            /* let map_blocks = [
                gm_maps.maps[ui_selector.map_editor_level_index].map
                    [ui_selector.map_editor_cursor.0 * 2][ui_selector.map_editor_cursor.1 * 2],
                gm_maps.maps[ui_selector.map_editor_level_index].map
                    [ui_selector.map_editor_cursor.0 * 2][ui_selector.map_editor_cursor.1 * 2 + 1],
                gm_maps.maps[ui_selector.map_editor_level_index].map
                    [ui_selector.map_editor_cursor.0 * 2 + 1][ui_selector.map_editor_cursor.1 * 2],
                gm_maps.maps[ui_selector.map_editor_level_index].map
                    [ui_selector.map_editor_cursor.0 * 2 + 1]
                    [ui_selector.map_editor_cursor.1 * 2 + 1],
            ];
            if [1, 2].contains(&ui_selector.map_editor_block) {
                let mut states: Vec<BlockState> = vec![];
                for (i, bool) in ui_selector.map_editor_blocks_inner
                    [ui_selector.map_editor_block - 1]
                    .iter()
                    .enumerate()
                {
                    if *bool && map_blocks[i] != 0 {
                        gm_maps.maps[ui_selector.map_editor_level_index].map
                            [ui_selector.map_editor_cursor.0 * 2 + (i / 2) as usize]
                            [ui_selector.map_editor_cursor.1 * 2 + (i % 2) as usize] = 0;
                        states.push(BlockState::Remove);
                    } else if !*bool {
                        if is_four(map_blocks[i]) {
                            
                        }
                        gm_maps.maps[ui_selector.map_editor_level_index].map
                            [ui_selector.map_editor_cursor.0 * 2 + (i / 2) as usize]
                            [ui_selector.map_editor_cursor.1 * 2 + (i % 2) as usize] =
                            ui_selector.map_editor_block;
                    }
                }
            } */

            /* let mut spawn: Vec<Block> = vec![];
            let mut delete = vec![];
            let mut change: Vec<Block> = vec![];
            if [1, 2].contains(&ui_selector.map_editor_block) {
                for (index, block) in map_blocks.iter().enumerate() {
                    if ui_selector.map_editor_blocks_inner[ui_selector.map_editor_block - 1][index]
                        && *block != 0
                    {
                        delete.push((
                            ui_selector.map_editor_cursor.0 * 2 + (index / 2) as usize,
                            ui_selector.map_editor_cursor.1 * 2 + (index % 2) as usize,
                        ));
                    } else if is_four(*block) {
                        delete.push((
                            ui_selector.map_editor_cursor.0 * 2,
                            ui_selector.map_editor_cursor.1 * 2,
                        ));
                        break;
                    }
                }
            } */
        }
    }
}
