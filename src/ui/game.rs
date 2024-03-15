use std::time::Duration;

use bevy::prelude::*;

use crate::{
    res::{
        Block, Bullet, Clear, Colider, Enemy, GameDirection, GameMapCollection, GameState, Moving,
        Player, UISelectInfo,
    },
    utils::{
        util::{a_star, path_to_move_direction, transform_to_pos},
        widget::{sprite, sprite_root},
    },
};

use super::{
    class::game_class::{class_sprite_bullet, class_sprite_panel},
    widget::wd_load_game_map,
};

pub fn setup_ui_game(
    commands: Commands,
    gm_map: Res<GameMapCollection>,
    ui_map_select: Res<UISelectInfo>,
    gm_state: Res<State<GameState>>,
) {
    sprite_root(class_sprite_panel, commands, Clear, |gc| {
        wd_load_game_map(gc, &gm_map, &ui_map_select, &gm_state);
    });
}

pub fn update_ui_game(mut query_player: Query<&mut Player>, key_event: Res<ButtonInput<KeyCode>>) {
    for mut player in query_player.iter_mut() {
        if let Some(keys) = player.keys_binding {
            if key_event.just_pressed(keys.up) {
                player.direction_stack.push(GameDirection::Up);
            } else if key_event.just_released(keys.up) {
                player.direction_stack.retain(|&x| x != GameDirection::Up);
            }
            if key_event.just_pressed(keys.down) {
                player.direction_stack.push(GameDirection::Down);
            } else if key_event.just_released(keys.down) {
                player.direction_stack.retain(|&x| x != GameDirection::Down);
            }
            if key_event.just_pressed(keys.left) {
                player.direction_stack.push(GameDirection::Left);
            } else if key_event.just_released(keys.left) {
                player.direction_stack.retain(|&x| x != GameDirection::Left);
            }
            if key_event.just_pressed(keys.right) {
                player.direction_stack.push(GameDirection::Right);
            } else if key_event.just_released(keys.right) {
                player
                    .direction_stack
                    .retain(|&x| x != GameDirection::Right);
            }
            if key_event.just_pressed(keys.fire) {
                player.fire = true;
            } else if key_event.just_released(keys.fire) {
                player.fire = false;
            }
        }
    }
}

pub fn update_ui_game_shoot(
    mut commands: Commands,
    mut query_player: Query<(&mut Player, &mut Transform, &mut TextureAtlas, &mut Moving)>,
    panel: Query<Entity, (With<Clear>, With<Sprite>)>,
    time: Res<Time>,
) {
    for (mut player, mut transform, mut layout, mut mov) in query_player.iter_mut() {
        if player.fire
            && (player.bullet.is_none()
                || (commands.get_entity(player.bullet.unwrap()).is_none()
                    && player.shoot_time >= Duration::from_millis(600)))
        {
            player.shoot_time = Duration::from_secs(0);
            for panel_entity in panel.iter() {
                commands.entity(panel_entity).with_children(|gc| {
                    let id = sprite(
                        class_sprite_bullet,
                        gc,
                        (
                            Moving::new(mov.direction, if player.level > 1 { 16. } else { 8. }),
                            Bullet::new(&player, &transform, &mov),
                        ),
                    );
                    player.bullet = Some(id);
                });
            }
        }
        update_player(
            transform.as_mut(),
            layout.as_mut(),
            mov.as_mut(),
            player.as_mut(),
        );
        if player.bullet.is_some() && player.shoot_time < Duration::from_millis(600) {
            player.shoot_time += time.delta();
        }
    }
}

pub fn update_player(
    transform: &mut Transform,
    layout: &mut TextureAtlas,
    tank: &mut Moving,
    player: &mut Player,
) {
    let idx = (layout.index + 1) % 2;
    if let Some(dir) = player.direction_stack.last() {
        tank.direction = *dir;
        match tank.direction {
            GameDirection::Up => {
                layout.index = idx;
                transform.translation.y += tank.speed as f32;
            }
            GameDirection::Down => {
                layout.index = 2 + idx;
                transform.translation.y -= tank.speed as f32;
            }
            GameDirection::Left => {
                layout.index = 4 + idx;
                transform.translation.x -= tank.speed as f32;
            }
            GameDirection::Right => {
                layout.index = 6 + idx;
                transform.translation.x += tank.speed as f32;
            }
        }
        if let Some(last_direction) = player.last_turn_direction {
            if last_direction != *dir {
                player.last_turn_direction = Some(*dir);
                let mod_v = 6.;
                let mod_y = transform.translation.y % mod_v;
                if mod_y > 4. {
                    transform.translation.y += mod_v - mod_y;
                } else {
                    transform.translation.y -= mod_y;
                }
                let mod_x = transform.translation.x % mod_v;
                if mod_x > 4. {
                    transform.translation.x += mod_v - mod_x;
                } else {
                    transform.translation.x -= mod_x;
                }
            }
        } else {
            player.last_turn_direction = Some(*dir);
        }
    }
}
//a星算法移动到home为目标的最短路径
pub fn update_ui_enemy(
    query_home: Query<(&Colider, &Transform), With<Block>>,
    mut query_enemy: Query<(&mut Player, &Transform, &mut Enemy)>,
    gm_map: Res<GameMapCollection>,
    select_info: Res<UISelectInfo>,
) {
    for (colider, transform) in query_home.iter() {
        if colider.index == 6 {
            for (mut player, enemy_transform, mut enemy) in query_enemy.iter_mut() {
                let mut search = false;
                if let Some(start) = enemy.start_search_pos {
                    if (enemy_transform.translation.x - start.0).abs() >= 24.
                        || (enemy_transform.translation.y - start.1).abs() >= 24.
                    {
                        enemy.start_search_pos =
                            Some((enemy_transform.translation.x, enemy_transform.translation.y));
                        search = true;
                    }
                }
                if enemy.start_search_pos.is_none() {
                    enemy.start_search_pos =
                        Some((enemy_transform.translation.x, enemy_transform.translation.y));
                    search = true;
                }
                if search {
                    let home_pos = transform_to_pos(transform);
                    let enemy_pos = transform_to_pos(enemy_transform);
                    if let Some(path) =
                        a_star(&gm_map.maps[select_info.map_index].map, enemy_pos, home_pos)
                    {
                        if let Some(dir) = path_to_move_direction(path) {
                            if player.direction_stack.is_empty() {
                                player.direction_stack.push(dir);
                            } else {
                                player.direction_stack[0] = dir;
                            }
                        }
                    }
                }
            }
        }
    }
}

//todo 简化代码
pub fn update_check_collision(
    mut query_movable: Query<(&mut Transform, &Colider, &Moving), With<Moving>>,
    query_colider: Query<(&Transform, &Colider), Without<Moving>>,
) {
    let mut iter = query_movable.iter_mut().collect::<Vec<_>>();
    for i in 0..iter.len() {
        let rh = iter[i].1.height / 2.0;
        let rw = iter[i].1.width / 2.0;
        for (st_transform, collider) in query_colider.iter() {
            let mv_top_edge = iter[i].0.translation.y + rh;
            let mv_bottom_edge = iter[i].0.translation.y - rh;
            let mv_left_edge = iter[i].0.translation.x - rw;
            let mv_right_edge = iter[i].0.translation.x + rw;
            if collider.is_container {
                match iter[i].2.direction {
                    GameDirection::Up => {
                        if mv_top_edge > collider.height / 2.0 {
                            iter[i].0.translation.y = collider.height / 2.0 - rh;
                        }
                    }
                    GameDirection::Down => {
                        if mv_bottom_edge < -collider.height / 2.0 {
                            iter[i].0.translation.y = -collider.height / 2.0 + rh;
                        }
                    }
                    GameDirection::Left => {
                        if mv_left_edge < -collider.width / 2.0 {
                            iter[i].0.translation.x = -collider.width / 2.0 + rw;
                        }
                    }
                    GameDirection::Right => {
                        if mv_right_edge > collider.width / 2.0 {
                            iter[i].0.translation.x = collider.width / 2.0 - rw;
                        }
                    }
                }
            } else {
                if !collider.is_container
                    && (mv_top_edge > st_transform.translation.y - collider.height / 2.0
                        && mv_bottom_edge < st_transform.translation.y + collider.height / 2.0
                        && mv_right_edge > st_transform.translation.x - collider.width / 2.0
                        && mv_left_edge < st_transform.translation.x + collider.width / 2.0)
                {
                    match iter[i].2.direction {
                        GameDirection::Up => {
                            iter[i].0.translation.y =
                                st_transform.translation.y - collider.height / 2.0 - rh;
                        }
                        GameDirection::Down => {
                            iter[i].0.translation.y =
                                st_transform.translation.y + collider.height / 2.0 + rh;
                        }
                        GameDirection::Left => {
                            iter[i].0.translation.x =
                                st_transform.translation.x + collider.width / 2.0 + rw;
                        }
                        GameDirection::Right => {
                            iter[i].0.translation.x =
                                st_transform.translation.x - collider.width / 2.0 - rw;
                        }
                    }
                }
            }
        }
        for j in i + 1..iter.len() {
            let jh = iter[j].1.height / 2.0;
            let jw = iter[j].1.width / 2.0;
            if iter[i].0.translation.y + rh > iter[j].0.translation.y - jh
                && iter[i].0.translation.y - rh < iter[j].0.translation.y + jh
                && iter[i].0.translation.x + rw > iter[j].0.translation.x - jw
                && iter[i].0.translation.x - rw < iter[j].0.translation.x + jw
            {
                match iter[i].2.direction {
                    GameDirection::Up => {
                        iter[i].0.translation.y = iter[j].0.translation.y - jh - rh;
                    }
                    GameDirection::Down => {
                        iter[i].0.translation.y = iter[j].0.translation.y + jh + rh;
                    }
                    GameDirection::Left => {
                        iter[i].0.translation.x = iter[j].0.translation.x + jw + rw;
                    }
                    GameDirection::Right => {
                        iter[i].0.translation.x = iter[j].0.translation.x - jw - rw;
                    }
                }
            }
        }
    }
}
//todo 简化代码
pub fn update_bullet_boom(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &mut Bullet, &Moving), With<Bullet>>,
    query_colider: Query<(Entity, &Transform, &Colider), Without<Bullet>>,
) {
    let mut iter = query.iter_mut().collect::<Vec<_>>();
    for i in 0..iter.len() {
        match iter[i].3.direction {
            GameDirection::Up => {
                iter[i].1.translation.y += iter[i].3.speed as f32;
            }
            GameDirection::Down => {
                iter[i].1.translation.y -= iter[i].3.speed as f32;
            }
            GameDirection::Left => {
                iter[i].1.translation.x -= iter[i].3.speed as f32;
            }
            GameDirection::Right => {
                iter[i].1.translation.x += iter[i].3.speed as f32;
            }
        }
        for (entity, transform, colider) in query_colider.iter() {
            let mv_top_edge = iter[i].1.translation.y + 6.;
            let mv_bottom_edge = iter[i].1.translation.y - 6.;
            let mv_left_edge = iter[i].1.translation.x - 6.;
            let mv_right_edge = iter[i].1.translation.x + 6.;
            if colider.is_container
                && match iter[i].3.direction {
                    GameDirection::Up => mv_top_edge > colider.height / 2.0,
                    GameDirection::Down => mv_bottom_edge < -colider.height / 2.0,
                    GameDirection::Left => mv_left_edge < -colider.width / 2.0,
                    GameDirection::Right => mv_right_edge > colider.width / 2.0,
                }
            {
                commands.entity(iter[i].0).despawn_recursive();
            } else if !colider.is_container && colider.index != 5 {
                if mv_top_edge > transform.translation.y - colider.height / 2.0
                    && mv_bottom_edge < transform.translation.y + colider.height / 2.0
                    && mv_right_edge > transform.translation.x - colider.width / 2.0
                    && mv_left_edge < transform.translation.x + colider.width / 2.0
                {
                    if !iter[i].2.boom {
                        iter[i].2.boom = true;
                        commands.entity(iter[i].0).despawn_recursive();
                    }
                    if colider.is_home() {
                        //game_over
                    } else {
                        commands.entity(entity).despawn_recursive();
                    }
                }
            }
        }
        for j in i + 1..iter.len() {
            if iter[i].1.translation.y + 6. > iter[j].1.translation.y - 6.
                && iter[i].1.translation.y - 6. < iter[j].1.translation.y + 6.
                && iter[i].1.translation.x + 6. > iter[j].1.translation.x - 6.
                && iter[i].1.translation.x - 6. < iter[j].1.translation.x + 6.
            {
                commands.entity(iter[i].0).despawn_recursive();
                commands.entity(iter[j].0).despawn_recursive();
            }
        }
    }
}
