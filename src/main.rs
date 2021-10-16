#![allow(unused)]

mod player;
mod enemy;
use bevy::{prelude::*, sprite::collide_aabb::collide};
use enemy::EnemyPlugin;
use player::PlayerPlugin;

const PLAYER_SPRITE: &str = "player.png";
const P_LASER_SPRITE: &str = "bluebeam.png";
const ENEMY_SPRITE: &str = "spiked ship 3.PNG";
const EXPLOSION: &str = "exp2.png";
const TIME_STEP: f32 =  1./60.;
const E_LASER_SPRITE: &str = "redbeam.png";
const RESPAWN_DELAY: f64 = 2.;
const MAX_ENEMIES: u32 = 2;
const MAX_FORMATION_MEMBERS: u32 = 2;

//region: Resources
pub struct Materials{
    player_mat: Handle<ColorMaterial>,
    laser: Handle<ColorMaterial>,
    enemy: Handle<ColorMaterial>,
    explosion: Handle<TextureAtlas>,
    e_laser: Handle<ColorMaterial>,
}
struct WinSize{
    w:f32,
    h:f32
}

struct ActiveEnemies(u32);
struct PlayerState{
    on: bool,
    last_shot: f64
}
impl Default for PlayerState{
    fn default()->Self{
        Self{
            on: false,
            last_shot: 0.
        }
    }
}
impl PlayerState{
    fn shot(&mut self, time:f64){
        self.on = false;
        self.last_shot = time;
    }
    fn spawned(&mut self){
        self.on =true;
        self.last_shot = 0.;
    }
}
//endregion: Resources

//region: Components
struct Player;
struct PlayerReadyFire(bool);
struct FromPlayer;

struct Laser;
struct Enemy;
struct FromEnemy;
struct Explosion;
struct ExplosionToSpawn(Vec3);
struct Speed(f32);
impl Default for Speed{
    fn default()-> Self{
        Self(500.) 
    }
}
//endregion: Components
fn main() {
    App::build()
    .insert_resource(ClearColor(Color::rgb(0.04,0.04,0.04)))
    .insert_resource(WindowDescriptor{
        title: "Rust Invaders!".to_string(),
        width: 600.0,
        height: 700.0,
        ..Default::default()
    })
    .insert_resource(ActiveEnemies(0))
    .add_plugins(DefaultPlugins) 
    .add_plugin(PlayerPlugin)
    .add_plugin(EnemyPlugin)
    .add_startup_system(setup.system())
    .add_system(laser_hit_enemy.system())
    .add_system(laser_hit_player.system())
    .add_system(explosion_to_spawn.system())
    .add_system(animate_exp.system())
    .run()
}

fn setup(
    mut commands:Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut windows: ResMut<Windows>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>
){
    let mut window = windows.get_primary_mut().unwrap();

    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let texture_handle = asset_server.load(EXPLOSION);
    let texture_atlas = TextureAtlas::from_grid(texture_handle, Vec2::new(250.,250.),4,4);
    //create main resources
    commands.insert_resource(Materials{
        player_mat: materials.add(asset_server.load(PLAYER_SPRITE).into()),
        laser: materials.add(asset_server.load(P_LASER_SPRITE).into()),
        enemy: materials.add(asset_server.load(ENEMY_SPRITE).into()),
        explosion: texture_atlases.add(texture_atlas),
        e_laser: materials.add(asset_server.load(E_LASER_SPRITE).into())
    });
    commands.insert_resource(WinSize{
        w: window.width(),
        h: window.height(),
    });

    //position window
    window.set_position(IVec2::new(0,0));
}

fn laser_hit_enemy(
    mut commands: Commands,
    mut laser_query: Query<(Entity, &Transform,&Sprite, (With<Laser>, With<FromPlayer>))>,
    mut enemy_query: Query<(Entity, &Transform,&Sprite, With<Enemy>)>,
    mut active_enemies: ResMut<ActiveEnemies>
){
    for (laser_e,laser_t, laser_s, _ ) in laser_query.iter_mut(){
        for (enemy_e,enemy_t,enemy_s, _) in enemy_query.iter_mut(){
            let laser_scale = Vec2::from(laser_t.scale);
            let enemy_scale = Vec2::from(enemy_t.scale);
            let collision = collide(
                laser_t.translation,
                laser_s.size * laser_scale,
                enemy_t.translation,
                enemy_s.size * enemy_scale,
            );

            if let Some(_) = collision{
                commands.entity(enemy_e).despawn();
                if active_enemies.0 > 0 {
                    active_enemies.0 -= 1;
                }

                commands.entity(laser_e).despawn();
                commands.spawn().insert(ExplosionToSpawn(enemy_t.translation.clone()));
            }
        }
    }
}
fn laser_hit_player(
    mut commands: Commands,
    mut player_state: ResMut<PlayerState>,
    time: Res<Time>,
    mut laser_query: Query<(Entity, &Transform,&Sprite), (With<Laser>, With<FromEnemy>)>,
    player_query: Query<(Entity, &Transform,&Sprite), (With<Player>)>,
    
){
    if let Ok((player_e,player_t,player_s)) =player_query.single(){
        
        let player_scale = Vec2::from(player_t.scale.abs());
        for (laser_e,laser_t, laser_s) in laser_query.iter_mut(){ 
            let laser_scale = Vec2::from(laser_t.scale);
            let collision = collide(
                laser_t.translation,
                laser_s.size * laser_scale,
                player_t.translation,
                player_s.size * player_scale,
            );
            
            if let Some(_) = collision{
                commands.entity(player_e).despawn();
                player_state.shot(time.seconds_since_startup());
                commands.entity(laser_e).despawn();
                commands.spawn().insert(ExplosionToSpawn(player_t.translation.clone()));
            }
        }
    }
}
fn explosion_to_spawn(
    mut commands: Commands,
    query: Query<(Entity,&ExplosionToSpawn)>,
    materials: Res<Materials>,
){
    for (explosion_spawn_e, explosion_to_spawn) in query.iter(){
        commands
        .spawn_bundle(SpriteSheetBundle{
            texture_atlas: materials.explosion.clone(),
            transform: Transform{
                translation: explosion_to_spawn.0,
                scale: Vec3::new(0.5,0.5, 1.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Explosion)
        .insert(Timer::from_seconds(0.05,true));

        commands.entity(explosion_spawn_e).despawn();
    }
}
fn animate_exp(
    mut commands: Commands,
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(Entity,&mut Timer, &mut TextureAtlasSprite,&Handle<TextureAtlas>,With<Explosion>)>,
){
    for (entity,mut timer, mut sprite, texture_atlas_handle, _) in query.iter_mut(){
        timer.tick(time.delta());
        if timer.finished(){
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index += 1;
            if sprite.index == texture_atlas.textures.len() as u32{

                commands.entity(entity).despawn()
            }
        }
    }
}