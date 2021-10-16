use std::f32::consts::PI;

use bevy::{core::FixedTimestep, prelude::*};
use rand::{Rng, thread_rng};

use crate::{ActiveEnemies, Enemy, FromEnemy, Laser, MAX_FORMATION_MEMBERS, Materials, Speed, TIME_STEP, WinSize};

pub struct EnemyPlugin;
#[derive(Default,Clone)]
struct Formation{
    start: (f32,f32),
    radius: (f32,f32),
    offset:(f32,f32),
    angle:f32,
    group_id: u32
}

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut AppBuilder){
        app
            .add_system(enemy_laser_movement.system())
            .add_system(enemy_movement.system())
            .add_system_set(
                SystemSet::new()
                    .with_run_criteria(FixedTimestep::step(1.))
                    .with_system(enemy_spawn.system()),
            ).add_system_set(
                SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.9))
                .with_system(enemy_fire.system()),
            );
    }
}
fn enemy_movement(
    time: Res<Time>, 
    mut query: Query<(&mut Transform, &Speed), With<Enemy>>
){
    let now = time.seconds_since_startup() as f32;

    for(mut t, speed) in query.iter_mut(){
        let max_dist = time.delta_seconds() * speed.0;
        let x_org = t.translation.x;
        let y_org = t.translation.y;

        //Def Elipse
        let (x_offset,y_offset) = (0., 100.);
        let (x_radius, y_radius) = (150.,100.);
        //Compute angle
        let angle = speed.0 * TIME_STEP * now % 360./PI;
        //Calc Destination
        let x_dst = x_radius * angle.cos() + x_offset;
        let y_dst = y_radius * angle.sin() + y_offset;
        //Calc distance
        let dx = x_org - x_dst;
        let dy = y_org - y_dst;
        let distance = (dx*dx + dy*dy).sqrt();
        let dist_ratio = if distance == 0.{
            0.
        }else{
            max_dist / distance
        };

        let x = x_org - dx * dist_ratio;
        let x = if dx>0.{x.max(x_dst) } else { x.min(x_dst) };
        let y = y_org - dy* dist_ratio;
        let y = if dy>0.{x.max(y_dst) } else { y.min(y_dst) };

        t.translation.x = x;
        t.translation.y = y;
    }
}
fn enemy_spawn(
    mut commands: Commands,
    mut active_enemies: ResMut<ActiveEnemies>,
    win_size: Res<WinSize>,
    materials: Res<Materials>
){
    if active_enemies.0 < 1{
        let mut rng = thread_rng();
        let w_span = win_size.w / 2. - 100.;
        let h_span = win_size.h /2. -100.;
        let x = rng.gen_range(-w_span..w_span) as f32;
        let y = rng.gen_range(-h_span..h_span) as f32;

        commands
            .spawn_bundle(SpriteBundle{
                material: materials.enemy.clone(),
                transform: Transform{
                    translation: Vec3::new(x,y,10.),
                    scale: Vec3::new(0.25,0.25, -1.), 
                    rotation: Quat::from_rotation_y(std::f32::consts::PI),
                    ..Default::default()
                },
                ..Default::default()
            })
            .insert(Enemy)
            .insert(Speed::default());

        active_enemies.0 += 1;
    }
}
fn enemy_fire(
    mut commands: Commands,
    materials: Res<Materials>,
    enemy_query: Query<&Transform, With<Enemy>>
){
    for &enemy_t in enemy_query.iter(){
        let x = enemy_t.translation.x;
        let y = enemy_t.translation.y;

        commands
            .spawn_bundle(
                SpriteBundle{
                    material: materials.e_laser.clone(),
                    transform: Transform{
                        translation: Vec3:: new(x,y - 15., 0.),
                        scale: Vec3::new(0.5, 0.5, 1.),
                        ..Default::default()
                    },
                    ..Default::default()
                }
            )
            .insert(Laser)
            .insert(FromEnemy)
            .insert(Speed::default());
    }
}
fn enemy_laser_movement(
    mut commands: Commands,
    time: Res<Time>,
    win_size: Res<WinSize>,
    mut query: Query<(Entity,&Speed,&mut Transform), (With<Laser>,With<FromEnemy>)>,
){
    for (entity,speed, mut t) in query.iter_mut(){
        t.translation.y -= speed.0 * time.delta_seconds();
        if t.translation.y < -win_size.h /2. -50.{
            commands.entity(entity).despawn();
        }
    }
}